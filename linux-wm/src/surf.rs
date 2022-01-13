use framebuffer::Framebuffer;
use log::info;
use common::{ARGBColor, BLACK, Rect};
use common::graphics::{ColorDepth, GFXBuffer};
use common::graphics::ColorDepth::{CD16, CD24, CD32};

pub struct Surf {
    fb:Framebuffer,
    buf:GFXBuffer,
    w:u32,
    h:u32,
}

impl Surf {
    pub(crate) fn make(fb: Framebuffer) -> Surf {
        let w = fb.var_screen_info.xres;
        let h = fb.var_screen_info.yres;
        let mut buf = match fb.var_screen_info.bits_per_pixel {
            16 => GFXBuffer::new(CD16(),w,h),
            24 => GFXBuffer::new(CD24(),w,h),
            32 => GFXBuffer::new(CD32(),w,h),
            _ => {
                panic!("unsupported resolution {}",fb.var_screen_info.bits_per_pixel);
            }
        };
        info!("made surface {}x{} at {}",w,h,fb.var_screen_info.bits_per_pixel);
        Surf { fb, w, h, buf }
    }
}

impl Surf {
    pub fn copy_from(&mut self, x: i32, y: i32, buf: &GFXBuffer) {
        self.buf.copy_from(x, y, buf);
    }
    pub fn rect(&mut self, rect: Rect, color: &ARGBColor) {
        for j in 0..rect.h {
            for i in 0..rect.w {
                self.buf.set_pixel_32argb((rect.x + i) as u32, (rect.y + j) as u32, color.as_32bit());
            }
        }
    }
    pub fn sync(&mut self) {
        self.fb.write_frame(&self.buf.data);
    }
    pub fn clear(&mut self) {
//         self.buf.fill_rect(Rect::from_ints(0, 0,
//             self.fb.var_screen_info.xres as i32,
//             self.fb.var_screen_info.yres as i32),
// BLACK);
    self.buf.fill_rect(Rect::from_ints(0, 0, 500, 500), BLACK);
    }
}
