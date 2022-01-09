use framebuffer::Framebuffer;
use common::{ARGBColor, BLACK, Rect};

pub struct Surf {
    fb:Framebuffer,
    frame: Vec<u8>,
}

impl Surf {
    pub(crate) fn make(fb: Framebuffer) -> Surf {
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
    pub fn rect(&mut self, rect: Rect, color: ARGBColor) {
        let ll = (self.fb.fix_screen_info.line_length/4) as i32;
        for j in 0..rect.h {
            for i in 0..rect.w {
                let n = (((rect.x+i) + (rect.y+j)*ll) * 4) as usize;
                self.frame[n] = color.b;
                self.frame[n + 1] = color.g;
                self.frame[n + 2] = color.r;
                self.frame[n + 3] = color.a;
            }
        }
    }
    pub fn sync(&mut self) {
        self.fb.write_frame(&self.frame);
    }
    pub fn clear(&mut self) {
        self.rect(Rect::from_ints(0, 0, self.fb.var_screen_info.xres as i32, self.fb.var_screen_info.yres as i32),BLACK);
    }
}
