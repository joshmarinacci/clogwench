extern crate framebuffer;

use std::fs::File;
use std::thread;
// use std::io::Write;
use std::process::{Child, Command};
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use ctrlc;
use env_logger;
use env_logger::Env;
use framebuffer::{Framebuffer, KdMode, VarScreeninfo};
use log::{debug, info, LevelFilter, log, warn};
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use structopt::StructOpt;

use common::{APICommand, ARGBColor, HelloWindowManager, IncomingMessage, Point, Rect, BLACK};
use common::APICommand::KeyDown;
use common::events::{KeyDownEvent, KeyCode};
use common_wm::{OutgoingMessage, start_wm_network_connection, WindowManagerState};
use surf::Surf;
use common::graphics::{ColorDepth, GFXBuffer};

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

    let cursor_image:GFXBuffer = GFXBuffer::from_png_file("../resources/cursor.png");

    if !args.disable_graphics {
        let pth = "/dev/fb0";
        let mut fb = Framebuffer::new(pth).unwrap();
        print_debug_info(&fb);
        let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
        let mut surf:Surf = Surf::make(fb);
        // surf.sync();
        let drawing_thread = make_drawing_thread(surf,stop.clone(),conn.rx_in, conn.tx_out.clone(), cursor_image);
    }

    let timeout_handle = start_timeout(stop.clone(),args.timeout);
    timeout_handle.join().unwrap();
    if !args.disable_graphics {
        let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
    }
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
                       tx_out: Sender<OutgoingMessage>,
                       cursor_image: GFXBuffer
) -> JoinHandle<()> {
    return thread::spawn(move ||{
        info!("render thread starting");
        let mut state = WindowManagerState::init();
        let mut cursor:Point = Point::init(0,0);
        // let mut test_buff = GFXBuffer::new(ColorDepth::CD24(),10,10);
        // test_buff.clear(&ARGBColor::new_rgb(0,0,0));

        // test_buff.fill_rect(Rect::from_ints(0,0,5,5),(ARGBColor::new_rgb(255,0,0)));
        // test_buff.fill_rect(Rect::from_ints(5,0,5,5),(ARGBColor::new_rgb(0,255,0)));
        // test_buff.fill_rect(Rect::from_ints(0,5,5,5),(ARGBColor::new_rgb(0,0,255)));
        // test_buff.fill_rect(Rect::from_ints(5,5,5,5),(ARGBColor::new_rgb(255,255,255)));
        for cmd in rx {
            if stop.load(Ordering::Relaxed) == true { break; }
            let mut redraw = false;
            match cmd.command {
                APICommand::AppConnectResponse(res) => {
                    info!("adding an app {}",res.app_id);
                    state.add_app(res.app_id);
                },
                APICommand::OpenWindowResponse(ow) => {
                    info!("adding a window to the app");
                    state.add_window(ow.app_id, ow.window_id, &ow.bounds);
                    state.set_focused_window(ow.window_id);
                    redraw = true;
                },
                APICommand::DrawRectCommand(dr) => {
                    info!("drawing a rect");
                    if let Some(mut win) = state.lookup_window(dr.window_id) {
                        win.backbuffer.fill_rect(dr.rect, dr.color);
                    }
                    redraw = true;
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
                    let bounds = Rect::from_ints(0,0,500,500);
                    let pt = bounds.clamp(&Point::init(mme.x,mme.y));
                    cursor.copy_from(pt);
                    redraw = true;
                },
                APICommand::MouseUp(mme) => {
                    // println!("mouse move {:?}",mme)
                },
                _ => {}
            }
            if redraw {
                //surf.clear();
                // surf.copy_from(0,0,&cursor_image);
                for win in state.window_list() {
                    surf.copy_from(win.bounds.x, win.bounds.y, &win.backbuffer)
                }
                surf.copy_from(cursor.x, cursor.y, &cursor_image);
                surf.sync();
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
    #[structopt(long)]
    disable_graphics:bool,
}

fn init_setup() -> Cli {
    let args:Cli = Cli::from_args();
    let loglevel = if args.debug { LevelFilter::Debug } else { LevelFilter::Error };

    // create file appender with target file path
    let logfile = FileAppender::builder()
        .build("log/output.log").expect("error setting up file appender");
    println!("logging to log/output.log");

    // make a config
    let config = Config::builder()
        //add the file appender
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        //now make it
        .build(Root::builder()
            .appender("logfile") // why do we need to mention logfile again?
            .build(loglevel)).expect("error setting up log file");

    log4rs::init_config(config).expect("error initing config");

    info!("running with args {:?}",args);
    return args;
}


fn setup_c_handler(stop: Arc<AtomicBool>) {
    // ctrlc::set_handler(move || {
    //     error!("control C pressed. stopping everything");
    //     stop.store(true, Ordering::Relaxed)
    // }).expect("error setting control C handler");
}
