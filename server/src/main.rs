extern crate framebuffer;

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::io::{self, prelude::*, BufReader};
use std::{fs, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use framebuffer::{Framebuffer, KdMode};
use serde::Deserialize;
use common::{APICommand, ARGBColor, DrawRectCommand};
use evdev::{Device, Key, EventType, InputEventKind};
use ctrlc;

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

// fn setup_listener() {
//mut surf: &mut Surf
fn setup_listener(arc: Arc<AtomicBool>) -> (JoinHandle<()>, Receiver<APICommand>) {
    fn handle_error(connection: io::Result<LocalSocketStream>) -> LocalSocketStream {
        match connection {
            Ok(val) => val,
            Err(error) => {
                eprintln!("\n");
                panic!("Incoming connection failed: {}", error);
            }
        }
    }
    let (tx, rx) = mpsc::channel();

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
            if arc.load(Ordering::Relaxed) {
                println!("its time to bail");
                break;
            }
            println!("server reading from socket");
            let cmd:APICommand =APICommand::deserialize(&mut de).unwrap();
            println!("server is getting results {:?}",cmd);
            tx.send(cmd).unwrap();
        }
    });
    (handle,rx)
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

struct Surf {
    fb:Framebuffer,
    frame: Vec<u8>,
}

impl Surf {
    fn make(fb: Framebuffer) -> Surf {
        let w = fb.var_screen_info.xres;
        let h = fb.var_screen_info.yres;
        let line_length = fb.fix_screen_info.line_length;
        let mut surf = Surf {
            fb: fb,
            frame: vec![0u8; (line_length * h) as usize]
        };
        surf
    }
}

impl Surf {
    fn rect(&mut self, x:i32, y:i32, w:i32, h:i32, color: ARGBColor) {
        let ll = (self.fb.fix_screen_info.line_length/4) as i32;
        for j in 0..h {
            for i in 0..w {
                let n = (((x+i) + (y+j)*ll) * 4) as usize;
                self.frame[n] = color.b;
                self.frame[n + 1] = color.g;
                self.frame[n + 2] = color.r;
                self.frame[n + 3] = color.a;
            }
        }
    }
    fn sync(&mut self) {
        self.fb.write_frame(&self.frame);
    }
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

fn dr(fb: &Framebuffer, frame: &mut Vec<u8>, x:i32, y:i32, w:i32, h:i32) {
}

fn sleep(ms:i32) {
    thread::sleep(Duration::from_millis(1000));
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

fn main() {
    let should_stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let ss2 = should_stop.clone();

    let mut keyboard = find_keyboard().expect("couldnt find the keyboard");
    let ss3 = should_stop.clone();
    thread::spawn(move || {
        let mut go = true;
        loop {
            if !go {
                break;
            }
            for ev in keyboard.fetch_events().unwrap() {
                // println!("{:?}", ev);
                // println!("type {:?}", ev.event_type());
                if let InputEventKind::Key(key) = ev.kind() {
                    println!("a key was pressed: {}",key.code());
                    if key == Key::KEY_ESC {
                        println!("trying to escape");
                        go = false;
                        ss3.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
    
    });

    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();
    print_debug_info(&framebuffer);
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    let mut surf:Surf = Surf::make(framebuffer);
    test_draw_rects(&mut surf);

    let (hand, rx) = setup_listener(should_stop.clone());
    // println!("now done here");
    let ch = start_process();
//    std::io::stdin().read_line(&mut String::new()).unwrap();
    thread::spawn(move ||{
        for cmd in rx {
            if should_stop.load(Ordering::Relaxed) {
                println!("it's time to stop");
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
                    println!("key down");
                },
            }
        }
    });

    // sleep(5000);
    // println!("now waiting for the client to die");
    // println!("server done");
    // ctrlc::set_handler(move || {
    //     let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
    //     println!("got control C");
    //     ss2.store(true, Ordering::Relaxed)
    // }).expect("error setting control C handler");
    hand.join().unwrap();
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}

