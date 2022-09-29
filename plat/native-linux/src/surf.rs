use framebuffer::Framebuffer;
use log::info;
use common::{ARGBColor, BLACK, Rect, Point};
use gfx::graphics::{GFXBuffer, PixelLayout};

pub struct Surf {
    fb:Framebuffer,
    pub(crate) buf:GFXBuffer,
    w:u32,
    h:u32,
}

impl Surf {
    pub(crate) fn make(fb: Framebuffer) -> Surf {
        let w = fb.var_screen_info.xres;
        let h = fb.var_screen_info.yres;
        let mut buf = match fb.var_screen_info.bits_per_pixel {
            16 => GFXBuffer::new(w,h, &PixelLayout::RGB565()),
            32 => GFXBuffer::new(w,h, &PixelLayout::ARGB()),
            _ => {
                panic!("unsupported resolution {}",fb.var_screen_info.bits_per_pixel);
            }
        };
        info!("made surface {}x{} px with  {} bits per pixel",w,h,fb.var_screen_info.bits_per_pixel);
        info!("buffer is {:?} {}", buf.layout, buf.bounds());
        Surf { fb, w, h, buf }
    }
}

impl Surf {
    pub fn draw_image(&mut self, dst_pos:&Point, src_bounds:&Rect, src_buf:&GFXBuffer ) {
        self.buf.draw_image(dst_pos, src_bounds, src_buf);
    }
    pub fn sync(&mut self) {
        self.fb.write_frame(&self.buf.data);
    }

}
