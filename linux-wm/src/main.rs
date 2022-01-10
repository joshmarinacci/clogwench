extern crate framebuffer;

use std::thread;
use std::io::Write;
use std::process::{Child, Command};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use ctrlc;
use env_logger;
use env_logger::Env;
use framebuffer::{Framebuffer, KdMode};
use log::{debug, error, info, log, warn};
use serde::Deserialize;
use structopt::StructOpt;
use uuid::Uuid;

use common::{APICommand, ARGBColor, HelloWindowManager, IncomingMessage, Point, Rect};
use common::APICommand::KeyDown;
use common::events::{KeyDownEvent, KeyCode};
use common_wm::{OutgoingMessage, start_wm_network_connection, WindowManagerState};
use surf::Surf;

mod surf;
mod input;

fn print_debug_info(framebuffer: &Framebuffer) {
    let s = String::from_utf8_lossy(&framebuffer.fix_screen_info.id);
    println!("id {}",s);
    println!("x {} y {}",framebuffer.fix_screen_info.xpanstep, framebuffer.fix_screen_info.ypanstep);
    println!("width {} height {}",framebuffer.var_screen_info.xres, framebuffer.var_screen_info.yres);
    println!("bits per pixel {}",framebuffer.var_screen_info.bits_per_pixel);
    println!("rotate {}",framebuffer.var_screen_info.rotate);
    println!("xoff {} yoff {}",framebuffer.var_screen_info.xoffset, framebuffer.var_screen_info.yoffset);
    println!("type {} {}", framebuffer.fix_screen_info.fb_type, framebuffer.fix_screen_info.type_aux);
    println!("accell {}", framebuffer.fix_screen_info.accel);
    println!("grayscale {}", framebuffer.var_screen_info.grayscale);

}

// create simple app to print text which is launched by the linux-wm
fn start_process() -> Child {
    println!("running some output");
    let mut list_dir = Command::new("../target/debug/drawrects")
        // .stdin(Stdio::null())
        // .stdout(Stdio::null())
        // .stdout(Stdio::inherit())
        .arg("/")
        // .env_clear()
        // .env("PATH", "/bin")
        .spawn()
        .expect("child process failed to start")
        ;
    println!("spawned it");
    list_dir
}

fn main() {
    let args:Cli = init_setup();
    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    //connect to input
    let mut keyboard = input::find_keyboard().expect("Couldn't find the keyboard");
    let mut mouse = input::find_mouse().expect("Couldn't find the mouse");

    //connect to network
    let conn = start_wm_network_connection(stop.clone())
        .expect("error connecting to the central server");
    //send hello window manager
    let msg = OutgoingMessage {
        recipient: Default::default(),
        command: APICommand::WMConnect(HelloWindowManager {
        })
    };
    conn.tx_out.send(msg).unwrap();

    let resp = conn.rx_in.recv().unwrap();
    let selfid = if let APICommand::WMConnectResponse(res) = resp.command {
        info!("got response back from the server {:?}",res);
        res.wm_id
    } else {
        panic!("did not get the window manager connect response. gah!");
    };


    //start input watchers
    input::setup_evdev_watcher(keyboard, stop.clone(), conn.tx_in.clone());
    input::setup_evdev_watcher(mouse, stop.clone(), conn.tx_in.clone());


    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();
    print_debug_info(&framebuffer);
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    let mut surf:Surf = Surf::make(framebuffer);
    //let ch = start_process();
    surf.sync();
    let drawing_thread = make_drawing_thread(surf,stop.clone(),conn.rx_in, conn.tx_out.clone());

    let timeout_handle = start_timeout(stop.clone(),args.timeout);
    timeout_handle.join().unwrap();
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
    info!("all done now");
}

fn start_timeout(stop: Arc<AtomicBool>, max_seconds:u32) -> JoinHandle<()> {
    return thread::spawn(move || {
        info!("timeout will end in {} seconds",max_seconds);
        let mut count = 0;
        loop {
            count = count + 1;
            if count > max_seconds {
                info!("timeout triggered");
                stop.store(true,Ordering::Relaxed);
            }
            thread::sleep(Duration::from_millis(1000));
            if stop.load(Ordering::Relaxed) == true { break; }
        }
    });
}

fn make_drawing_thread(mut surf: Surf,
                       stop: Arc<AtomicBool>,
                       rx: Receiver<IncomingMessage>,
                       tx_out: Sender<OutgoingMessage>
) -> JoinHandle<()> {
    return thread::spawn(move ||{
        info!("render thread starting");
        let mut state = WindowManagerState::init();
        for cmd in rx {
            if stop.load(Ordering::Relaxed) == true { break; }
            match cmd.command {
                APICommand::AppConnectResponse(res) => {
                    info!("adding an app {}",res.app_id);
                    state.add_app(res.app_id);
                },
                APICommand::OpenWindowResponse(ow) => {
                    info!("adding a window to the app");
                    state.add_window(ow.app_id, ow.window_id, &ow.bounds);
                },
                APICommand::DrawRectCommand(cm) => {
                    surf.rect(cm.rect,cm.color);
                    surf.sync();
                },
                APICommand::KeyUp(ku) => {
                    // println!("key up");
                },
                APICommand::KeyDown(kd) => {
                    // println!("key down {}",kd.key);
                    match kd.key {
                        KeyCode::ESC => {
                            stop.store(true, Ordering::Relaxed);
                        },
                        _ => {
                            info!("key down");
                            //send key to the currently focused window
                            if let Some(winid) = state.get_focused_window() {
                                if let Some(win) = state.lookup_window(winid.clone()) {
                                    let msg = OutgoingMessage {
                                        recipient: win.owner,
                                        command: KeyDown(KeyDownEvent {
                                            app_id: win.owner,
                                            window_id: win.id,
                                            original_timestamp: 0,
                                            key: kd.key
                                        })
                                    };
                                    tx_out.send(msg).unwrap();
                                }
                            }
                        }
                    }
                },
                APICommand::MouseDown(mme) => {
                    let pt = Point::init(mme.x, mme.y);
                    info!("mouse down at {:?}",pt);
                    if let Some(win) = state.pick_window_at(pt) {
                        debug!("found a window at {:?}", pt);
                        state.set_focused_window(win.id);
                    }
                },
                APICommand::MouseMove(mme) => {
                    let color = ARGBColor{
                        r: 0,
                        g: 255,
                        b: 255,
                        a: 255
                    };
                    let bounds = Rect::from_ints(0,0,500,500);
                    let pt = bounds.clamp(&Point::init(mme.x,mme.y));
                    // //surf.clear();
                    let cursor = Rect::from_ints(pt.x,pt.y,10,10);
                    surf.rect(cursor, color);
                    surf.sync();
                    //println!("mouse move {:?},{:?}",(mme.x/10),(mme.y/10))
                },
                APICommand::MouseUp(mme) => {
                    // println!("mouse move {:?}",mme)
                },
                _ => {}
            }
        }
        info!("render thread stopping");
    });
}


#[derive(StructOpt, Debug)]
#[structopt(name = "test-wm", about = "simulates receiving and sending linux-wm events")]
struct Cli {
    #[structopt(short, long)]
    debug:bool,
    #[structopt(short, long, default_value="60")]
    timeout:u32,
}

fn init_setup() -> Cli {
    let args:Cli = Cli::from_args();
    let loglevel = if args.debug { "debug"} else { "error"};
    env_logger::Builder::from_env(Env::default().default_filter_or(loglevel)).init();
    info!("running with args {:?}",args);
    return args;
}


fn setup_c_handler(stop: Arc<AtomicBool>) {
    // ctrlc::set_handler(move || {
    //     error!("control C pressed. stopping everything");
    //     stop.store(true, Ordering::Relaxed)
    // }).expect("error setting control C handler");
}
