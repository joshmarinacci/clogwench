use framebuffer::Framebuffer;
use common::ARGBColor;

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
    pub fn rect(&mut self, x:i32, y:i32, w:i32, h:i32, color: ARGBColor) {
        let ll = (self.fb.fix_screen_info.line_length/4) as i32;
        for j in 0..h {
            for i in 0..w {
                let n = (((x+i) + (y+j)*ll) * 4) as usize;
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
        let black = ARGBColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255
        };
        self.rect(0, 0, self.fb.var_screen_info.xres as i32, self.fb.var_screen_info.yres as i32, black);
    }
}
