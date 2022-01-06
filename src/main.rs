use std::error::Error;
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

    //let location = (x+vinfo.xoffset) * (vinfo.bits_per_pixel/8) + (y+vinfo.yoffset) * finfo.line_length;
    //  *((uint32_t*)(fbp + location)) = pixel;

    let result5 = unsafe { ioctl(0, KDSETMODE as _, KD_TEXT)};
    if result5 == -1 { panic!("error fscreen info") }


    Ok(())
}
