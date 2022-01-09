extern crate framebuffer;

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::io::{self, BufReader};
use std::{fs, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use framebuffer::{Framebuffer, KdMode};
use serde::Deserialize;
use common::{APICommand, ARGBColor, KeyDownEvent, MouseMoveEvent};
use evdev::{Device, EventType, InputEventKind, Key, AbsoluteAxisType, RelativeAxisType};
use ctrlc;
use surf::Surf;
use structopt::StructOpt;
use log::{info, warn, error,log};
use env_logger;
use env_logger::Env;

mod network;
mod surf;


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

// create simple app to print text which is launched by the server
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

fn find_keyboard() -> Option<evdev::Device> {
    let mut devices = evdev::enumerate().collect::<Vec<_>>();
    devices.reverse();
    for (i, d) in devices.iter().enumerate() {
        if d.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
            println!("found keyboard device {}",d.name().unwrap_or("Unnamed device"));
            return devices.into_iter().nth(i);
        }
    }
    None
}

fn find_mouse() -> Option<evdev::Device> {
    let mut devices = evdev::enumerate().collect::<Vec<_>>();
    devices.reverse();
    for (i, d) in devices.iter().enumerate() {
        for typ in d.supported_events().iter() {
            println!("   type {:?}",typ);
        }
        if d.supported_events().contains(EventType::RELATIVE) {
            println!("found a device with relative input {}", d.name().unwrap_or("unnamed device"));
            return devices.into_iter().nth(i);
        }
        if d.supported_events().contains(EventType::ABSOLUTE) {
            println!("found a device with absolute input: {}", d.name().unwrap_or("Unnamed device"));
            return devices.into_iter().nth(i);
        }
        // if d.supported_relative_axes().map_or(false, |axes| axes.contains(RelativeAxisType::REL_X)) {
        //     println!("found a device with relative input: {}", d.name().unwrap_or("Unnamed device"));
        //     return devices.into_iter().nth(i);
        // }
    }
    None
}

fn main() {
    let args:Cli = init_setup();
    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    let (tx, rx) = mpsc::channel::<APICommand>();
    let mut keyboard = find_keyboard().expect("Couldn't find the keyboard");
    let mut mouse = find_mouse().expect("Couldn't find the mouse");
    setup_evdev_watcher(keyboard, stop.clone(),tx.clone());
    setup_evdev_watcher(mouse, stop.clone(),tx.clone());

    network::start_network_server(stop.clone(), tx);

    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();
    print_debug_info(&framebuffer);
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    let mut surf:Surf = Surf::make(framebuffer);
    //let ch = start_process();
    surf.sync();
    let drawing_thread = make_drawing_thread(surf,stop.clone(),rx);

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


fn make_drawing_thread(mut surf: Surf, stop: Arc<AtomicBool>, rx: Receiver<APICommand>) -> JoinHandle<()> {
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
                    
                    if kd.key == 1 { //wait for the ESC key
                        stop.store(true, Ordering::Relaxed);
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

fn setup_evdev_watcher(mut device: Device, stop: Arc<AtomicBool>, tx: Sender<APICommand>) {
    thread::spawn(move || {
        let mut cx = 0;
        let mut cy = 0;
        loop {
            if stop.load(Ordering::Relaxed) == true {
                println!("keyboard thread stopping");
                break;
            }
            for ev in device.fetch_events().unwrap() {
                // println!("{:?}", ev);
                println!("type {:?}", ev.event_type());
                match ev.kind() {
                    InputEventKind::Key(key) => {
                        println!("   evdev:key {}",key.code());
                        let cmd = APICommand::KeyDown(KeyDownEvent{
                            original_timestamp:0,
                            key:key.code() as i32,
                        });
                        tx.send(cmd).unwrap()
                    },
                    InputEventKind::RelAxis(rel) => {
                        println!("mouse event {:?} {}",rel, ev.value());
                        match rel {
                            RelativeAxisType::REL_X => cx += ev.value(),
                            RelativeAxisType::REL_Y => cy += ev.value(),
                            _ => {
                                println!("unknown relative axis type");
                            }
                        }
                        println!("cursor {} , {}",cx, cy);
                        let cmd = APICommand::MouseMove(MouseMoveEvent{
                            original_timestamp:0,
                            button:0,
                            x:cx,
                            y:cy
                        });
                        tx.send(cmd).unwrap()
                    },
                    InputEventKind::AbsAxis(abs) => {
                        // println!("abs event {:?} {:?}",ev.value(), abs);
                        match abs {
                            AbsoluteAxisType::ABS_X => cx = ev.value()/10,
                            AbsoluteAxisType::ABS_Y => cy = ev.value()/10,
                            _ => {
                                println!("unknown aboslute axis type")
                            }
                        }
                        let cmd = APICommand::MouseMove(MouseMoveEvent{
                            original_timestamp:0,
                            button:0,
                            x:cx,
                            y:cy
                        });
                        tx.send(cmd).unwrap()
                        //stop.store(true,Ordering::Relaxed);
                    },
                    _ => {}
                }
            }
        }
    });
}


#[derive(StructOpt, Debug)]
#[structopt(name = "test-server", about = "simulates receiving and sending server events")]
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
