use std::sync::mpsc::Sender;
use common::{ARGBColor, IncomingMessage, Rect, BLACK, Point};
use common::graphics::GFXBuffer;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};

use framebuffer::{Framebuffer, KdMode, VarScreeninfo};
use log::info;

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
    let v = &framebuffer.var_screen_info;
    info!("id {}",s);
    info!("x {} y {}",framebuffer.fix_screen_info.xpanstep, framebuffer.fix_screen_info.ypanstep);
    info!("width {} height {}",v.xres, v.yres);
    info!("width {} height {}", v.width, v.height);
    info!("{:?}",v);
    info!("bits per pixel {}",framebuffer.var_screen_info.bits_per_pixel);
    println!("rotate {}",framebuffer.var_screen_info.rotate);
    println!("xoff {} yoff {}",framebuffer.var_screen_info.xoffset, framebuffer.var_screen_info.yoffset);
    println!("type {} {}", framebuffer.fix_screen_info.fb_type, framebuffer.fix_screen_info.type_aux);
    println!("accell {}", framebuffer.fix_screen_info.accel);
    println!("grayscale {}", framebuffer.var_screen_info.grayscale);

    info!("red bif {:?}",v.red);
    info!("gre bif {:?}",v.green);
    info!("blu bif {:?}",v.blue);
    info!("tra bif {:?}",v.transp);

}

impl Plat {
    pub fn get_screen_bounds(&self) -> Rect {
        self.screen_size
    }
    pub fn service_input(&mut self) {}
    pub fn service_loop(&mut self) {
        self.surf.sync();
    }
    pub fn fill_rect(&mut self, rect: Rect, color: &ARGBColor) {
        self.surf.buf.fill_rect(rect,color);
    }
    pub fn draw_image(&mut self, dst_pos:&Point, src_bounds:&Rect, src_buf:&GFXBuffer ) {
        self.surf.draw_image(dst_pos, src_bounds, src_buf);
    }
    pub fn clear(&mut self) {
        self.surf.buf.clear(&BLACK);
    }
    pub fn shutdown(&mut self) {

    }
    pub fn register_image2(&mut self, img:&GFXBuffer) {

    }
    pub fn unregister_image2(&mut self, img:&GFXBuffer) {

    }
}

