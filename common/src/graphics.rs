/*
- make screen and graphics abstraction with separate testing and linux impulse
	- Build the rect abstraction l. Use it for drawing rects
	    and window bounds and deciding which
	    window gets the mouse events.
	- create a virtual surface used for testing. then the
	    linux-wm can have multiple gfx implementations.
	    common screen and surface impls.
 */

use crate::{ARGBColor, Rect};

pub enum ColorDepth {
    CD16(),
    CD24(),
    CD32(),
}

pub struct GFXBuffer {
    bitdepth:ColorDepth,
    width:u32,
    height:u32,
    pub data: Vec<u8>,
}

fn pixel_16_packed_to_24_rgb(src:u16, dst:u32) {

}

fn pixel_24_rgb_to_16_packed(src:u32) -> u16 {
    //split
    let r = src & 0x00FF0000 >> 16;
    let g = src & 0x0000FF00 >> 8;
    let b = src & 0x000000FF >> 0;
    let rp = r >> 3;
    let gp = g >> 2;
    let bp = b >> 3;
    let fbal:u16 = (((rp as u32) << (11)) | ((gp as u32) << 5) | ((bp as u32) << 0)) as u16;
    return fbal;
}

impl GFXBuffer {
    pub fn copy_from(&mut self, x:i32, y:i32, src: &GFXBuffer) {
        for i in 0..src.width {
            for j in 0..src.height {
                let v:u32 = src.get_pixel_32argb(i, j);
                self.set_pixel_32argb(i+(x as u32), j+(y as u32), v);
            }
        }
    }

    pub fn fill_rect(&mut self, bounds: Rect, color: ARGBColor) {
        for i in bounds.x .. (bounds.x+bounds.w) {
            for j in bounds.y .. (bounds.y+bounds.h) {
                self.set_pixel_32argb(i as u32, j as u32, color.as_32bit());
            }
        }
    }

    pub fn get_pixel_32argb(&self, x: u32, y: u32) -> u32 {
        match self.bitdepth {
            ColorDepth::CD16() => {
                let n = (x + y * (self.width as u32)) as usize;
                let packed_color:u16 = ((self.data[n*2+0] as u16) << 8) | (self.data[n*2+1] as u16);
                ARGBColor::from_16bit(packed_color).as_32bit()
            }
            ColorDepth::CD24() => {
                let n = (x + y * (self.width as u32)) as usize;
                ARGBColor::new_rgb(self.data[n*3+0], self.data[n*3+1], self.data[n*3+2]).as_32bit()
            }
            ColorDepth::CD32() => {
                let n = (x + y * (self.width as u32)) as usize;
                ARGBColor::new_argb(self.data[n*3+0], self.data[n*3+1], self.data[n*3+2], self.data[n*3+3]).as_32bit()
            }
        }
    }
    pub fn set_pixel_32argb(&mut self, x: u32, y: u32, v: u32) {
        match self.bitdepth {
            ColorDepth::CD16() => {
                let vv = ARGBColor::from_24bit(v).as_16bit();
                let n = (x + y * (self.width as u32)) as usize;
                self.data[n*2+0] = ((vv & 0xFF00) >> 8) as u8;
                self.data[n*2+1] = ((vv & 0x00FF) >> 0) as u8;
            }
            ColorDepth::CD24() => {}
            ColorDepth::CD32() => {}
        }
    }
}

impl GFXBuffer {
    pub(crate) fn get_vec_pixel_32argb(&self, x: i32, y: i32) -> Vec<u8> {
        match self.bitdepth {
            ColorDepth::CD16() => {
                let n = (x + y * (self.width as i32)) as usize;
                let packed_color:u16 = ((self.data[n*2+0] as u16) << 8) | (self.data[n*2+1] as u16);
                let c = ARGBColor::from_16bit(packed_color);
                let mut data:Vec<u8> = Vec::new();
                data.push(c.a);
                data.push(c.r);
                data.push(c.g);
                data.push(c.b);
                return data;
            }
            ColorDepth::CD24() => {
                let n = (x + y * (self.width as i32)) as usize;
                let mut data:Vec<u8> = Vec::new();
                data.push(255);
                data.push(self.data[n*3+0]);
                data.push(self.data[n*3+1]);
                data.push(self.data[n*3+2]);
                return data;
            }
            ColorDepth::CD32() => {
                let n = (x + y * (self.width as i32)) as usize;
                let mut data:Vec<u8> = Vec::new();
                data.push(self.data[n*4+0]);
                data.push(self.data[n*4+1]);
                data.push(self.data[n*4+2]);
                data.push(self.data[n*4+3]);
                return data;
            }
        }
    }
}

impl GFXBuffer {
    pub fn clear(&mut self, color: &ARGBColor) {
        for i in 0..self.width {
            for j in 0..self.height {
                //let n = self.xy_to_n(i,j);
                let n = (i + j * self.width) as usize;
                self.set_pixel_n(n,color)
            }
        }
    }
    fn set_pixel_n(&mut self, n: usize, color: &ARGBColor) {
        match self.bitdepth {
            ColorDepth::CD16() => {
                // return (r<<vinfo->red.offset) | (g<<vinfo->green.offset) | (b<<vinfo->blue.offset);
                let val:u16 = color.as_16bit();//color.r << 11 | color.g << 5 | color.b << 0;
                println!("16bit color is {}",val);
                let mask1 = 0x0000ff00;
                let mask2 = 0x000000ff;
                self.data[n*2 + 0] = ((val & mask1) >> 8) as u8;
                self.data[n*2 + 1] = ((val & mask2) >> 0) as u8;
            }
            ColorDepth::CD24() => {
                self.data[n*3 + 0] = color.r;
                self.data[n*3 + 1] = color.g;
                self.data[n*3 + 2] = color.b;
            }
            ColorDepth::CD32() => {
                self.data[n*4 + 0] = color.a;
                self.data[n*4 + 1] = color.r;
                self.data[n*4 + 2] = color.g;
                self.data[n*4 + 3] = color.b;
            }
        }
    }
}

impl GFXBuffer {
    pub fn new(bitdepth:ColorDepth, width:u32, height:u32) -> GFXBuffer {
        let byte_length = match bitdepth {
            ColorDepth::CD16() => {width*height*2}
            ColorDepth::CD24() => {width*height*3}
            ColorDepth::CD32() => {width*height*4}
        };
        let data = vec![0u8; byte_length as usize];
        GFXBuffer {
            bitdepth,
            data,
            width,
            height
        }
    }
}

// rs.fillRect(rect,color)
// rs:fillRect(surf,rect,color)
// rs.copy(src:Surface,
// rs.clear(color)
// rs.setPixel(color,pt)
// rs.getPixelAsCD16()
// rs.getPixelAsCD24()
// rs.getPixelAsCD32(),
// rs.getColorDepth()->ColorDepth
// bytes = rs.as_CD16bytes() // something


//create 32bit color buffer for each window so we get alpha
// let winbuf = GFXBuffer::new(CD32(),500,500);
//create screen buffer in 16bit to match the real screen
// let screenbuff = GFXBuffer:new(CD16(),1024,768);
//copy window buffers to screen buffer w/ conversion
// screenbuff.copy(winbuff:GFXBuffer, winbuff.size():Size, dst:Point)
//draw to window buffer using ARGB
// winbuff.fillRect(rect,color)
//draw to screen buffer using ARGB w/ conversion
// screenbuff.fillRect(rect,color)
//sync screenbuffer to real framebuffer. expose as something that the linux crate can consume w/o being platform specific.
// FrameBuffer.copyFrom(screenbuff.as_bytes())

//tests
//fill small 24bit buffer with yellow (no alpha). confirm it has the right bits using getPixelAs24()
//copy small 24bit buffer to small 16bit buffer, confirm it got the right bits. repeat with several colors
//copy small buffer to bigger buffer, confirm it dioesn't crash.
//copy big buffer to smaller buffer. confirm it doesn't crash.
//copy buffer to another but offset so it tries to draw outside the dst area. confirm it doesn't crash.

#[cfg(test)]
mod tests {
    use crate::{ARGBColor, BLACK, WHITE};
    use crate::graphics::ColorDepth::{CD16, CD24, CD32};
    use crate::graphics::GFXBuffer;

    #[test]
    fn color_checks() {
        let color = ARGBColor::new_rgb(255, 255, 255);
        let val:u16 = color.as_16bit();
        let n:u32 = 2;
        // println!("16bit color is {:#024b} {}",val, n.pow(16)-1);
        assert_eq!(val as u32,n.pow(16)-1);

        //convert red from 24 to 16 bit
        assert_eq!(ARGBColor::new_rgb(255, 0, 0).as_16bit(), 0b1111100000000000);
        //convert red from 16 to 24 bit
        assert_eq!(ARGBColor::from_16bit(0b11111_000000_00000 as u16).r, ARGBColor::new_rgb(0xF8, 0, 0).r);
        assert_eq!(ARGBColor::from_16bit(0b11111_000000_00000 as u16).as_24bit(),0b00000000_11111000_00000000_00000000);
        // let vv = ARGBColor::from_24bit(0b11111111_00000000_00000000 as u32).as_32bit();
        // println!(" vv is {:x}",vv);
        assert_eq!(ARGBColor::from_24bit(0b11111111_00000000_00000000 as u32).as_32bit(), 0xFFFF0000);
        assert_eq!(ARGBColor::from_24bit(0x00FF00 as u32).as_16bit(), 0b00000_111111_00000);
                   // 0b00000000_11111111_00000000_00000000 as u32);
    }

    #[test]
    fn set_colors() {
        //fill small 16bit buffer with yellow. calculate manually what it should be converted to. check that value is correct. test again with several other colors
        let colors = [
            BLACK,
            WHITE,
            ARGBColor::new_rgb(0, 255, 255), //yellow
            ARGBColor::new_rgb(255, 0, 255), //magenta
            ARGBColor::new_rgb(255, 0, 0), //red
            ];

        for set_color in &colors {
            println!("color is {:x}",set_color.as_32bit());
            let mut buf = GFXBuffer::new(CD24(),2,2);
            buf.clear(&set_color);
            let color = buf.get_vec_pixel_32argb(0, 0);
            assert_eq!(color,set_color.as_vec());
        }
        for set_color in &colors {
            let mut buf = GFXBuffer::new(CD32(),2,2);
            buf.clear(&set_color);
            let color = buf.get_vec_pixel_32argb(0, 0);
            assert_eq!(color,set_color.as_vec());
        }
        // println!("color is {:?}",color);
        // println!("data is {:?}",buf.data);
    }

    #[test]
    fn check_24_to_16() {
        let mut buf24 = GFXBuffer::new(CD24(), 2, 2);
        let green = ARGBColor::new_rgb(0, 255, 0);
        buf24.clear(&green);
        let mut buf16 = GFXBuffer::new(CD16(), 2, 2);
        buf16.copy_from(0,0,&buf24);
        {
            let c1 = buf16.get_pixel_32argb(1, 1);
            let c2 = buf24.get_pixel_32argb(1, 1);
            assert_eq!(c1, 0b11111111_00000000_11111100_00000000);
        }
    }
}
