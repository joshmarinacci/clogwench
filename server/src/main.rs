extern crate framebuffer;

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::io::{self, BufReader, prelude::*};
use std::{fs, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use framebuffer::{Framebuffer, KdMode};
use serde::Deserialize;
use common::{APICommand, ARGBColor, DrawRectCommand, KeyDownEvent};
use evdev::{Device, EventType, InputEventKind, Key};
use ctrlc;
use surf::Surf;

mod surf;

fn fill_rect(frame: &mut Vec<u8>, w:u32, h:u32, line_length: u32, bytespp: u32) {
    for (r, line) in frame.chunks_mut(line_length as usize).enumerate() {
        for (c, p) in line.chunks_mut(bytespp as usize).enumerate() {
            let x0 = (c as f32 / w as f32) * 3.5 - 2.5;
            let y0 = (r as f32 / h as f32) * 2.0 - 1.0;

            let mut it = 0;
            let max_it = 200;

            let mut x = 0.0;
            let mut y = 0.0;

            while x * x + y * y < 4.0 && it < max_it {
                let xtemp = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xtemp;
                it += 1;
            }

            // p[0] = (125.0 * (it as f32 / max_it as f32)) as u8;
            // p[1] = (255.0 * (it as f32 / max_it as f32)) as u8;
            // p[2] = (75.0 * (it as f32 / max_it as f32)) as u8;
            p[0] = 0; //B
            p[1] = 0; //G
            p[2] = 128; //R
        }
    }
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

fn setup_listener(arc: Arc<AtomicBool>, tx:Sender<APICommand>) -> JoinHandle<()> {
    fn handle_error(connection: io::Result<LocalSocketStream>) -> LocalSocketStream {
        match connection {
            Ok(val) => val,
            Err(error) => {
                eprintln!("\n");
                panic!("Incoming connection failed: {}", error);
            }
        }
    }

    fs::remove_file("/tmp/teletype.sock");
    let listener =
        LocalSocketListener::bind("/tmp/teletype.sock").expect("failed to set up server");
    eprintln!("Teletype server listening for connections.");
    let handle = thread::spawn(move ||{
        let mut conn = listener
            .incoming()
            .next()
            .map(handle_error)
            .map(BufReader::new)
            .unwrap();
        let mut de = serde_json::Deserializer::from_reader(conn);
        loop {
            if arc.load(Ordering::Relaxed) == true {
                println!("socket thread stopping");
                break;
            }
            println!("server reading from socket");
            let cmd:APICommand =APICommand::deserialize(&mut de).unwrap();
            println!("server is getting results {:?}",cmd);
            tx.send(cmd).unwrap();
        }
    });
    handle
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
        .expect("ls failed to start")
        ;
    println!("spawned it");
    list_dir
}

fn test_draw_rects(mut surf: &mut Surf) {
    surf.rect(10, 10, 10, 10, ARGBColor{
        r: 255,
        g: 0,
        b: 0,
        a: 255
    });
    surf.rect(10, 30, 10, 10, ARGBColor{
        r: 0,
        g: 255,
        b: 0,
        a: 255
    });
    surf.rect(10, 50, 10, 10, ARGBColor{
        r: 0,
        g: 0,
        b: 255,
        a: 255
    });
    surf.sync();
}

fn sleep(ms:u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn find_keyboard() -> Option<evdev::Device> {
    let devices = evdev::enumerate().collect::<Vec<_>>();
    for (i, d) in devices.iter().enumerate() {
        if d.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
            return devices.into_iter().nth(i);
        }
    }
    None
}

fn find_mouse() -> Option<evdev::Device> {
    let devices = evdev::enumerate().collect::<Vec<_>>();
    for (i, d) in devices.iter().enumerate() {
        if d.supported_events().contains(EventType::RELATIVE) {
        // if d.supported_keys().map_or(false, |keys| keys.contains(Key::BTN_0)) {
            println!("found a device with relative input");
            return devices.into_iter().nth(i);
        }
    }
    None
}

fn main() {
    let should_stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let ss2 = should_stop.clone();

    let (tx, rx) = mpsc::channel::<APICommand>();

    let mut keyboard = find_keyboard().expect("Couldn't find the keyboard");
    let mut mouse = find_mouse().expect("Couldn't find the mouse");
    setup_evdev_watcher(keyboard, should_stop.clone(),tx.clone());
    setup_evdev_watcher(mouse, should_stop.clone(),tx.clone());

   let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();
   print_debug_info(&framebuffer);
   let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
   let mut surf:Surf = Surf::make(framebuffer);
   test_draw_rects(&mut surf);

    let hand = setup_listener(should_stop.clone(), tx);
    let ch = start_process();
    let ss4 = should_stop.clone();
    thread::spawn(move ||{
        for cmd in rx {
            if ss4.load(Ordering::Relaxed) == true {
                println!("render thread stopping");
                break;
            }
            match cmd {
                APICommand::OpenWindowCommand(cm) => println!("open window"),
                APICommand::DrawRectCommand(cm) => {
                    println!("draw rect");
                   surf.rect(cm.x,cm.y,cm.w,cm.h, cm.color);
                   surf.sync();
                },
                APICommand::KeyUp(ku) => {
                    println!("key up");
                },
                APICommand::KeyDown(kd) => {
                    println!("key down {}",kd.key);
                    if kd.key == 1 { //wait for the ESC key
                        ss4.store(true, Ordering::Relaxed);
                    }
                },
                APICommand::MouseDown(mme) => {
                    println!("mouse move {:?}",mme)
                },
                APICommand::MouseMove(mme) => {
                    println!("mouse move {:?}",mme)
                },
                APICommand::MouseUp(mme) => {
                    println!("mouse move {:?}",mme)
                },
            }
        }
    });

    // control c handler
    ctrlc::set_handler(move || {
        ss2.store(true, Ordering::Relaxed)
    }).expect("error setting control C handler");

    //timeout thread
    let timeout_handle = thread::spawn(move || {
        let mut count = 0;
        loop {
            count = count + 1;
            if count > 15 {
                should_stop.store(false,Ordering::Relaxed);
            }
            println!("watchdog sleeping for 1000");
            sleep(1000);
            if should_stop.load(Ordering::Relaxed) == true {
                println!("render thread stopping");
                break;
            }
        }
    });
    timeout_handle.join().unwrap();
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
    println!("all done now");
}

fn setup_evdev_watcher(mut device: Device, stop: Arc<AtomicBool>, tx: Sender<APICommand>) {
    thread::spawn(move || {
        loop {
            if stop.load(Ordering::Relaxed) == true {
                println!("keyboard thread stopping");
                break;
            }
            for ev in device.fetch_events().unwrap() {
                // println!("{:?}", ev);
                // println!("type {:?}", ev.event_type());
                match ev.kind() {
                    InputEventKind::Key(key) => {
                        let cmd = APICommand::KeyDown(KeyDownEvent{
                            original_timestamp:0,
                            key:key.code() as i32,
                        });
                        tx.send(cmd).unwrap();
                    },
                    InputEventKind::RelAxis(rel) => {
                        println!("mouse event");
                    },
                    _ => {}
                }
                // if let InputEventKind::Key(key) = ev.kind() {
                    //     if key == Key::KEY_ESC {
                    //         println!("trying to escape");
                    //         go = false;
                    //         ss3.store(true, Ordering::Relaxed);
                    //     }
                    //    println!("a key was pressed: {}",key.code());
                // }
            }
        }
    });
}

