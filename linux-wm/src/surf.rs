use framebuffer::Framebuffer;
use common::{ARGBColor, BLACK, Rect};
use common_wm::{BackBuffer};

pub struct Surf {
    fb:Framebuffer,
    data: Vec<u8>,
    w:u32,
    h:u32,
    bpp:u8,
}

impl Surf {
    pub(crate) fn make(fb: Framebuffer) -> Surf {
        let w = fb.var_screen_info.xres;
        let h = fb.var_screen_info.yres;
        let line_length = fb.fix_screen_info.line_length;
        let mut surf = Surf {
            fb: fb,
            w:w,
            h:h,
            bpp:4,
            data: vec![0u8; (line_length * h) as usize]
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
                self.data[n] = color.b;
                self.data[n + 1] = color.g;
                self.data[n + 2] = color.r;
                self.data[n + 3] = color.a;
            }
        }
    }
    pub fn sync(&mut self) {
        self.fb.write_frame(&self.data);
    }
    pub fn clear(&mut self) {
        self.rect(Rect::from_ints(0, 0, self.fb.var_screen_info.xres as i32, self.fb.var_screen_info.yres as i32),BLACK);
    }
    pub fn copy_from(&mut self, sx: i32, sy: i32, src: &BackBuffer) {
        let x = sx as u32;
        let y = sy as u32;
        println!("copying buffer {}x{} to self at {},{}", src.w, src.h, x,y);
        for j in 0..src.h {
            for i in 0..src.w {
                let src_n =  src.xy_to_n(i,j);
                let dst_n = self.xy_to_n((x + i) as u32, (y + j) as u32);
                for o in 0..4 {
                    self.data[dst_n+ o] = src.data[src_n+ o]
                }
            }
        }
    }
    pub fn xy_to_n(&self, x: u32, y: u32) -> usize {
        return ((x + self.w * y) * (self.bpp as u32)) as usize;
    }
}
