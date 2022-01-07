extern crate framebuffer;

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::io::{self, prelude::*, BufReader};
use std::thread;
use std::time::Duration;
use framebuffer::{Framebuffer, KdMode};
use serde::Deserialize;
use common::{APICommand, DrawRectCommand};

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
fn setup_listener(mut framebuffer: Framebuffer) {
    let w = framebuffer.var_screen_info.xres;
    let h = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let bytespp = framebuffer.var_screen_info.bits_per_pixel / 8;

    fn handle_error(connection: io::Result<LocalSocketStream>) -> LocalSocketStream {
        match connection {
            Ok(val) => val,
            Err(error) => {
                eprintln!("\n");
                panic!("Incoming connection failed: {}", error);
            }
        }
    }

    let listener =
        LocalSocketListener::bind("/tmp/teletype.sock").expect("failed to set up server");
    eprintln!("Teletype server listening for connections.");
    let mut conn = listener
        .incoming()
        .next()
        .map(handle_error)
        .map(BufReader::new)
        .unwrap();
    // let mut our_turn = false;
    let mut buffer = String::new();
    let mut de = serde_json::Deserializer::from_reader(conn);

    let mut frame = vec![0u8; (line_length * h) as usize];
    fill_rect(&mut frame,w,h, line_length, bytespp);

    loop {
        println!("server reading from socket");
        let cmd:APICommand =APICommand::deserialize(&mut de).unwrap();
        println!("server is getting results {:?}",cmd);
        let _ = framebuffer.write_frame(&frame);
        match cmd {
            APICommand::OpenWindowCommand(cm) => println!("open window"),
            APICommand::DrawRectCommand(cm) => println!("draw redct"),
        }
    }
}


// create simple app to print text which is launched by the server
fn start_process() {
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
}

struct Surf {
    fb:Framebuffer,
    frame: Vec<u8>,
}

impl Surf {
    fn rect(&mut self, x:i32, y:i32, w:i32, h:i32) {
        let ll = (self.fb.fix_screen_info.line_length/4) as i32;
        for j in 0..h {
            for i in 0..w {
                let n = (((x+i) + (y+j)*ll) * 4) as usize;
                self.frame[n] = 0;
                self.frame[n + 1] = 255;
                self.frame[n + 2] = 0;
                self.frame[n + 3] = 255;
            }
        }
    }
    fn sync(&mut self) {
        self.fb.write_frame(&self.frame);
    }
}

fn test_draw_rects(mut fb: Framebuffer) {
    let w = fb.var_screen_info.xres;
    let h = fb.var_screen_info.yres;
    let line_length = fb.fix_screen_info.line_length;
    let mut surf = Surf {
        fb: fb,
        frame: vec![0u8; (line_length * h) as usize]
    };
    // let mut frame = fb.read_frame();
    surf.rect(10, 10, 10, 10);
    surf.rect( 100, 100, 10, 10);
    surf.sync();
}

fn dr(fb: &Framebuffer, frame: &mut Vec<u8>, x:i32, y:i32, w:i32, h:i32) {
}

fn sleep(ms:i32) {
    thread::sleep(Duration::from_millis(1000));

}


fn main() {
    // start_process();
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();
    print_debug_info(&framebuffer);
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    test_draw_rects(framebuffer);
    // setup_listener();
    // setup_listener(framebuffer);
    // std::io::stdin().read_line(&mut String::new()).unwrap();
    sleep(5000);
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
    println!("server done");
}

