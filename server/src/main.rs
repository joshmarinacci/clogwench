extern crate framebuffer;

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::io::{self, prelude::*, BufReader};
use std::time::Duration;
use framebuffer::{Framebuffer, KdMode};
use serde::Deserialize;
use common::DrawRectCommand;

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

fn setup_listener() {
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
    loop {
        println!("server reading from socket");
        let rect:DrawRectCommand = DrawRectCommand::deserialize(&mut de).unwrap();
        println!("server is getting results {:?}",rect);
        // io::stdout()
        //     .write_all(buffer.as_ref())
        //     .expect("failed to write line to stdout");
        // buffer.clear();
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


fn main() {
    start_process();
    setup_listener();
    // let (tx,rx):(Sender<DrawRectCommand>,Receiver<DrawRectCommand>) = mpsc::channel();
    // let child = thread::spawn(move ||{
    //     loop {
    //         println!("child sending draw rect");
    //         let dr = DrawRectCommand {
    //             x: 1,
    //             y: 2,
    //             w: 3,
    //             h: 4
    //         };
    //         tx.send(dr).unwrap();
    //         thread::sleep(Duration::from_millis(1000))
    //     }
    // });
    // println!("server is waiting for child to end");
    // for received in rx {
    //     println!("got {:?}",received)
    // }
    //child.join().unwrap();
    println!("server done")

    //start_process();
    /*
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let w = framebuffer.var_screen_info.xres;
    let h = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let bytespp = framebuffer.var_screen_info.bits_per_pixel / 8;
    print_debug_info(&framebuffer);

    let mut frame = vec![0u8; (line_length * h) as usize];
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    fill_rect(&mut frame,w,h, line_length, bytespp);
    let _ = framebuffer.write_frame(&frame);
    std::io::stdin().read_line(&mut String::new()).unwrap();
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();*/
}

