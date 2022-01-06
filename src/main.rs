extern crate framebuffer;

use framebuffer::{Framebuffer, KdMode};

//Algorithm copied from:
//https://en.wikipedia.org/wiki/Mandelbrot_set
fn main() {
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let w = framebuffer.var_screen_info.xres;
    let h = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let bytespp = framebuffer.var_screen_info.bits_per_pixel / 8;
    println!("x {} y {}",framebuffer.fix_screen_info.xpanstep, framebuffer.fix_screen_info.ypanstep);
    println!("width {} height {}",framebuffer.var_screen_info.xres, framebuffer.var_screen_info.yres);
    println!("bits per pixel {}",framebuffer.var_screen_info.bits_per_pixel);
    println!("rotate {}",framebuffer.var_screen_info.rotate);
    println!("xoff {} yoff {}",framebuffer.var_screen_info.xoffset, framebuffer.var_screen_info.yoffset);


    let mut frame = vec![0u8; (line_length * h) as usize];

    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();

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
            p[1] = 0;
            p[2] = 128; //R
        }
    }

    let _ = framebuffer.write_frame(&frame);

    println!("waiting");
    std::io::stdin().read_line(&mut String::new()).unwrap();
    println!("got return");
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
    println!("cleaning up");
}
/*use std::error::Error;
use std::{thread, time};
use std::fs::OpenOptions;
use std::io;
use std::os::unix::io::AsRawFd;
use crate::cool_bindings::{FixScreeninfo, VarScreeninfo};

use libc::ioctl;
use memmap::{MmapMut, MmapOptions};

mod cool_bindings;

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOPUT_VSCREENINFO: libc::c_ulong = 0x4601;
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
const KDSETMODE: libc::c_ulong = 0x4B3A;
const KD_TEXT: libc::c_ulong = 0x00;
const KD_GRAPHICS: libc::c_ulong = 0x01;


fn write_frame(frame:&[u8]) {

}

fn main() -> Result<(), io::Error>{

    println!("switching to graphics mode");
    // let tty = OpenOptions::new()
    //     .read(true).write(true).open("/dev/tty0")?;
    let result0 = unsafe { ioctl(0, KDSETMODE as _, KD_GRAPHICS)};
    if result0 == -1 { panic!("error switching tty info") }

    println!("opening the graphics device fb0");
    //open fb0
    let device = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/fb0").unwrap();

    //get the variable screen info
    let mut vinfo:VarScreeninfo = Default::default();
    let result1 = unsafe { ioctl(device.as_raw_fd(), FBIOGET_VSCREENINFO as _, &mut vinfo)};
    if result1 == -1 { panic!("error vscreen info") }
    vinfo.grayscale = 0;
    vinfo.bits_per_pixel = 32;
    let result1a = unsafe { ioctl(device.as_raw_fd(), FBIOPUT_VSCREENINFO as _, &mut vinfo)};
    if result1a == -1 { panic!("error vscreen info a") }
    let result1c = unsafe { ioctl(device.as_raw_fd(), FBIOGET_VSCREENINFO as _, &mut vinfo)};
    if result1c == -1 { panic!("error vscreen info c") }


    let mut finfo:FixScreeninfo = Default::default();
    let result2 = unsafe { ioctl(device.as_raw_fd(), FBIOGET_FSCREENINFO as _, &mut finfo)};
    if result2 == -1 { panic!("error fscreen info") }



    //get the fixed screen info

    let frame_length = (finfo.line_length & vinfo.yres_virtual) as usize;
    let frame = unsafe { MmapOptions::new().len(frame_length).map_mut(&device)};

    println!("width is ${}", vinfo.xres);
    println!("height is ${}", vinfo.yres);
    //println!("vinfo is ${}",vinfo);


    let mut frame = vec![0u8; (finfo.line_length * vinfo.yres) as usize];
    for x in frame.iter_mut() {
        *x = 128
    }

    let ten_millis = time::Duration::from_millis(2000);
    thread::sleep(ten_millis);
    //let _ = write_frame(&frame);

    println!("created a frame");

    //let location = (x+vinfo.xoffset) * (vinfo.bits_per_pixel/8) + (y+vinfo.yoffset) * finfo.line_length;
    //  *((uint32_t*)(fbp + location)) = pixel;

    let result5 = unsafe { ioctl(0, KDSETMODE as _, KD_TEXT)};
    if result5 == -1 { panic!("error fscreen info") }

    println!("successfully left graphics mode back to text");
    println!("width is {}", vinfo.xres);
    println!("height is {}", vinfo.yres);

    Ok(())
}
*/
