extern crate framebuffer;

use framebuffer::{Framebuffer, KdMode};

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

fn main() {
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
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}
