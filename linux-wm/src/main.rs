extern crate framebuffer;

use std::{fs, thread};
use std::io::{self, BufReader};
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use ctrlc;
use env_logger;
use env_logger::Env;
use evdev::{AbsoluteAxisType, Device, EventType, InputEventKind, Key, RelativeAxisType};
use framebuffer::{Framebuffer, KdMode};
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use log::{error, info, log, warn};
use serde::Deserialize;
use structopt::StructOpt;

use common::{APICommand, ARGBColor};
use common::events::{KeyDownEvent, KeyUpEvent, KeyCode};
use surf::Surf;

mod network;
mod surf;
mod input;

pub struct App {
    connection:TcpStream,
    pub receiver_handle: JoinHandle<()>,
}


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

fn sleep(ms:u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn main() {
    let args:Cli = init_setup();
    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    let (tx, rx) = mpsc::channel::<APICommand>();
    let mut keyboard = input::find_keyboard().expect("Couldn't find the keyboard");
    let mut mouse = input::find_mouse().expect("Couldn't find the mouse");
    input::setup_evdev_watcher(keyboard, stop.clone(), tx.clone());
    input::setup_evdev_watcher(mouse, stop.clone(), tx.clone());

    let app_list: Arc<Mutex<Vec<App>>> = Arc::new(Mutex::new(Vec::new()));

    network::start_network_server(stop.clone(), tx, app_list.clone());

    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();
    print_debug_info(&framebuffer);
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    let mut surf:Surf = Surf::make(framebuffer);
    //let ch = start_process();
    surf.sync();
    let drawing_thread = make_drawing_thread(surf,stop.clone(),rx,app_list.clone());

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
    rx: Receiver<APICommand>,
    app_list: Arc<Mutex<Vec<App>>>
) -> JoinHandle<()> {
    return thread::spawn(move ||{
        for cmd in rx {
            if stop.load(Ordering::Relaxed) == true {
                println!("render thread stopping");
                break;
            }
            match cmd {
                APICommand::OpenWindowCommand(cm) => {
                    // println!("open window")
                },
                APICommand::DrawRectCommand(cm) => {
                    surf.rect(cm.x,cm.y,cm.w,cm.h, cm.color);
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
                            let cmd2: APICommand = APICommand::KeyDown(KeyDownEvent {
                                original_timestamp: kd.original_timestamp,
                                key: kd.key,
                            });
                            let data = serde_json::to_string(&cmd2).unwrap();
                            let mut v = app_list.lock().unwrap();
                            for app in v.iter_mut() {
                                app.connection.write_all(data.as_ref()).expect("failed to send rect");
                            }
                        }
                    }
                },
                APICommand::MouseDown(mme) => {
                    // println!("mouse move {:?}",mme)
                },
                APICommand::MouseMove(mme) => {
                    let color = ARGBColor{
                        r: 0,
                        g: 255,
                        b: 255,
                        a: 255
                    };
                    //surf.clear();
                    let mut x = mme.x;
                    if x < 0  {x = 0;}
                    if x > 500 {x = 500;}
                    let mut y = mme.y;
                    if y < 0 {y = 0;}
                    if y > 500 {y = 500;}
                    surf.rect(x,y,10,10, color);
                    surf.sync();
                    //println!("mouse move {:?},{:?}",(mme.x/10),(mme.y/10))
                },
                APICommand::MouseUp(mme) => {
                    // println!("mouse move {:?}",mme)
                },
            }
        }
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
    ctrlc::set_handler(move || {
        error!("control C pressed. stopping everything");
        stop.store(true, Ordering::Relaxed)
    }).expect("error setting control C handler");
}