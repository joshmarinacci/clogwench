use std::sync::mpsc::Sender;
use common::{ARGBColor, IncomingMessage, Rect, BLACK};
use common::graphics::GFXBuffer;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};

use framebuffer::{Framebuffer, KdMode, VarScreeninfo};

use surf::Surf;
mod surf;
mod input;


pub struct Plat {
    sender:Sender<IncomingMessage>,
    surf:Surf,
    screen_size:Rect,
}

pub fn make_plat<'a>(stop:Arc<AtomicBool>, sender: Sender<IncomingMessage>) -> Result<Plat, String> {
    let mut keyboard = input::find_keyboard().expect("Couldn't find the keyboard");
    let mut mouse = input::find_mouse().expect("Couldn't find the mouse");


    let pth = "/dev/fb0";
    let mut fb = Framebuffer::new(pth).unwrap();
    let screen_size = Rect::from_ints(0,0,fb.var_screen_info.xres as i32, fb.var_screen_info.yres as i32);
    print_debug_info(&fb);
    //let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    let mut surf:Surf = Surf::make(fb);
    surf.buf.clear(&ARGBColor::new_rgb(0,255,200));
    surf.sync();

    input::setup_evdev_watcher(keyboard, stop.clone(), sender.clone(), screen_size);
    input::setup_evdev_watcher(mouse, stop.clone(), sender.clone(), screen_size);


    return Ok(Plat {
        sender: sender,
        surf:surf,
        screen_size: screen_size,
    });
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

impl Plat {
    pub fn get_screen_bounds(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0
        }
    }
    pub fn service_input(&mut self) {}
    pub fn service_loop(&mut self) {
        self.surf.sync();
    }
    pub fn fill_rect(&mut self, rect: Rect, color: &ARGBColor) {
        self.surf.buf.fill_rect(rect,color.clone());
    }
    pub fn draw_rect(&mut self, rect: Rect, color: &ARGBColor, width: i32) {
        self.surf.buf.draw_rect(rect,color.clone(),width);
    }
    pub fn draw_image(&mut self, x: i32, y: i32, img: &GFXBuffer) {
        self.surf.copy_from(x, y, img);
    }
    pub fn clear(&mut self) {
        self.surf.buf.clear(&BLACK);
    }
    pub fn shutdown(&mut self) {

    }
    pub fn register_image2(&mut self) {

    }
}

