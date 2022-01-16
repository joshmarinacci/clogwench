/*
- make screen and graphics abstraction with separate testing and linux impulse
	- Build the rect abstraction l. Use it for drawing rects
	    and window bounds and deciding which
	    window gets the mouse events.
	- create a virtual surface used for testing. then the
	    linux-wm can have multiple gfx implementations.
	    common screen and surface impls.
 */

use std::fs::File;
use png;
use uuid::Uuid;
use crate::{ARGBColor, Rect};
use crate::graphics::ColorDepth::{CD24, CD32};

pub enum ColorDepth {
    CD16(),
    CD24(),
    CD32(),
}

pub struct GFXBuffer {
    pub bitdepth:ColorDepth,
    pub id:Uuid,
    pub width:u32,
    pub height:u32,
    pub data: Vec<u8>,
}

impl GFXBuffer {
    pub fn from_png_file(path: &str) -> GFXBuffer {
        let decoder = png::Decoder::new(File::open(path).unwrap());
        let mut reader = decoder.read_info().unwrap();
        println!("loading bytes {}", reader.output_buffer_size());
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        println!("size {}x{} bitdepth={:?} colortype={:?}",info.width, info.height, info.bit_depth, info.color_type);
        let bytes = &buf[..info.buffer_size()];
        let mut gfx = GFXBuffer::new(CD32(), info.width, info.height);
        for j in 0..info.height {
            for i in 0..info.width {
                let n = (i + j*info.width) as usize;
                gfx.set_pixel_argb(i,j, bytes[n*4+3],bytes[n*4+0],bytes[n*4+1],bytes[n*4+2]);
            }
        }
        return gfx
    }
}


impl GFXBuffer {
    pub fn copy_from(&mut self, x:i32, y:i32, src: &GFXBuffer) {
        for i in 0..src.width {
            for j in 0..src.height {
                let v = src.get_pixel_vec_argb(i,j);
                self.set_pixel_vec_argb(i+(x as u32), j+(y as u32), &v);
            }
        }
    }

    pub fn fill_rect(&mut self, bounds: Rect, color: ARGBColor) {
        let v = color.to_argb_vec();
        for i in bounds.x .. (bounds.x+bounds.w) {
            for j in bounds.y .. (bounds.y+bounds.h) {
                self.set_pixel_vec_argb(i as u32,j as u32,&v);
            }
        }
    }
    pub fn draw_rect(&mut self, bounds: Rect, color: ARGBColor, size: i32) {
        let v = color.to_argb_vec();
        for i in bounds.x .. (bounds.x+bounds.w) {
            for j in 0..size {
                self.set_pixel_vec_argb(i as u32, (bounds.y+j) as u32,&v);
                self.set_pixel_vec_argb(i as u32, (bounds.y+bounds.h-j) as u32,&v);
            }
        }
        for j in bounds.y .. (bounds.y+bounds.h) {
            for i in 0..size {
                self.set_pixel_vec_argb( (bounds.x+i) as u32, j as u32,&v);
                self.set_pixel_vec_argb( (bounds.x+bounds.w-i) as u32, j as u32, &v);
            }
        }
    }

    pub fn get_pixel_vec_argb(&self, x:u32, y:u32) -> Vec<u8>{
        let mut v:Vec<u8> = vec![0,0,0,0];
        if x >= self.width || y >= self.height {
            println!("error. pixel {},{} out of bounds {}x{}",x,y,self.width,self.height);
            return v;
        }
        match self.bitdepth {
            ColorDepth::CD16() => {
                let n = (x + y * (self.width as u32)) as usize;
                let packed_color:u16 = ((self.data[n*2+0] as u16) << 8) | (self.data[n*2+1] as u16);
                // return ARGBColor::from_16bit(packed_color).to_argb_vec();
                let r:u8 = (((packed_color & 0b11111_000000_00000) >> 11) << 3) as u8;
                let g:u8 = (((packed_color & 0b00000_111111_00000) >> 5)  << 2) as u8;
                let b:u8 = (((packed_color & 0b00000_000000_11111) >> 0)  << 3) as u8;
                v[0] = 255;
                v[1] = r;
                v[2] = g;
                v[3] = b;
            }
            ColorDepth::CD24() => {
                let n = (x + y * (self.width as u32)) as usize;
                v[0] = self.data[n*3+0];
                v[1] = self.data[n*3+1];
                v[2] = self.data[n*3+2];
            }
            ColorDepth::CD32() => {
                let n = (x + y * (self.width as u32)) as usize;
                v[0] = self.data[n*4+0];
                v[1] = self.data[n*4+1];
                v[2] = self.data[n*4+2];
                v[3] = self.data[n*4+3];
            }
        }
        return v
    }
    pub fn set_pixel_vec_argb(&mut self, x:u32, y:u32, v:&Vec<u8>) {
        if x >= self.width || y >= self.height {
            println!("error. pixel {},{} out of bounds {}x{}",x,y,self.width,self.height);
            return;
        }
        match self.bitdepth {
            ColorDepth::CD16() => {
                let vv = ARGBColor::from_argb_vec(v).as_16bit();
                let n = (x + y * (self.width as u32)) as usize;
                self.data[n*2+0] = ((vv & 0x00FF) >> 0) as u8;
                self.data[n*2+1] = ((vv & 0xFF00) >> 8) as u8;
            }
            ColorDepth::CD24() => {
                let n = (x + y * (self.width as u32)) as usize;
                self.data[n*3+0] = v[0];
                self.data[n*3+1] = v[1];
                self.data[n*3+2] = v[2];
            }
            ColorDepth::CD32() => {
                let n = (x + y * (self.width as u32)) as usize;
                self.data[n*4+0] = v[0];
                self.data[n*4+1] = v[1];
                self.data[n*4+2] = v[2];
                self.data[n*4+3] = v[3];
            }
        }
    }
    pub fn set_pixel_argb(&mut self, x:u32, y:u32, a:u8,r:u8,g:u8,b:u8) {
        self.set_pixel_vec_argb(x,y,&vec![a,r,g,b]);
    }
}

impl GFXBuffer {
    pub fn clear(&mut self, color: &ARGBColor) {
        match self.bitdepth {
            ColorDepth::CD16() => {
                let c1 = color.as_16bit();
                let p1 = ((0xFF00 | c1) >> 8) as u8;
                let p2 = ((0x00FF | c1) >> 0) as u8;
                //make a complete row
                let mut row:Vec<u8> = vec![];
                for i in 0..self.width {
                    row.push(p1);
                    row.push(p2);
                }
                for chunk in self.data.chunks_exact_mut((self.width*2) as usize) {
                    chunk.copy_from_slice(&*row);
                }

            }
            CD24() => {
                let v = color.as_vec();
                let r = v[1];
                let g = v[2];
                let b = v[3];
                let mut row:Vec<u8> = vec![];
                for i in 0..self.width {
                    row.push(r);
                    row.push(g);
                    row.push(b);
                }
                for chunk in self.data.chunks_exact_mut((self.width*3) as usize) {
                    chunk.copy_from_slice(&*row);
                }
            }
            CD32() => {
                //impl1
                let v = color.as_vec();
                let a = v[0];
                let r = v[1];
                let g = v[2];
                let b = v[3];

                let vv = &v;
                let mut row:Vec<u8> = vec![];
                for i in 0..self.width {
                    row.push(r);
                    row.push(g);
                    row.push(b);
                    row.push(a);
                }
                for chunk in self.data.chunks_exact_mut((self.width*4) as usize) {
                    chunk.copy_from_slice(&*row);
                }
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
            height,
            id: Uuid::new_v4(),
        }
    }
}

pub fn draw_test_pattern(buf:&mut GFXBuffer) {
    for j in 0..buf.height {
        for i in 0..buf.width {
            let v = (i*4) as u8;
            if j == 0 || j == (buf.height-1) {
                buf.set_pixel_argb(i,j,255,255,255,255);
                continue;
            }
            if j < (buf.height/4)*1 {
                buf.set_pixel_argb(i,j,255,v,0,0);
                continue;
            }
            if j < (buf.height/4)*2 {
                buf.set_pixel_argb(i,j,255,0,v,0);
                continue;
            }
            if j < (buf.height/4)*3 {
                buf.set_pixel_argb(i,j,255,0,0,v);
                continue;
            }
            if j < (buf.height/4)*4 {
                buf.set_pixel_argb(i,j,v,128,128,128);
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufWriter;
    use std::path::Path;
    use std::time::Instant;
    use crate::{ARGBColor, BLACK, WHITE};
    use crate::graphics::ColorDepth::{CD16, CD24, CD32};
    use crate::graphics::{draw_test_pattern, GFXBuffer};

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
            let color = buf.get_pixel_vec_argb(0, 0);
            assert_eq!(color,set_color.as_vec());
        }
        for set_color in &colors {
            let mut buf = GFXBuffer::new(CD32(),2,2);
            buf.clear(&set_color);
            let color = buf.get_pixel_vec_argb(0, 0);
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
            let c1 = buf16.get_pixel_vec_argb(1, 1);
            assert_eq!(c1, vec![255,0,248,0]);//0b11111111_00000000_11111100_00000000);
        }
    }

    #[test]
    fn check_32_to_32() {
        let mut buf = GFXBuffer::new(CD32(),2,2);
        buf.set_pixel_argb(0,0, 255,255,254,253);
        // buf.set_pixel_32argb(0,0, ARGBColor::new_rgb(255,254,253).as_32bit());
        // print!("{:x} vs {:x}", ARGBColor::new_rgb(255,254,253).as_32bit(), buf.get_pixel_32argb(0,0));
        assert_eq!(buf.get_pixel_vec_argb(0,0),vec![255,255,254,253]);
        // assert_eq!(buf.get_pixel_32argb(0,0),0xFFFFFEFD);
    }

    #[test]
    fn try_test_pattern() {
        let mut buf = GFXBuffer::new(CD32(),64,64);
        draw_test_pattern(&mut buf);

        // buf = GFXBuffer::new(CD32(),32,32);
        // draw_test_pattern(&mut buf);
        export_to_png(&buf);
    }

    fn export_to_png(buf: &GFXBuffer) {
        let path = Path::new(r"test.png");
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, buf.width, buf.height); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        let mut data:Vec<u8> = vec![];
        for j in 0..buf.height {
            for i in 0..buf.width {
                let px = buf.get_pixel_vec_argb(i,j);
                // println!("{},{}  {:x}",i,j, buf.get_pixel_32argb(i,j));
                data.push(px[1]); //R
                data.push(px[2]); //G
                data.push(px[3]); //B
                data.push(255); //A
            }
        }
        writer.write_image_data(&data).unwrap(); // Save

    }

    #[test]
    fn drawing_speed() {
        let mut buf = GFXBuffer::new(CD32(),1360,768);
        let now = Instant::now();
        for i in 0..50 {
            buf.clear(&BLACK);
        }
        println!("32bit elapsed {} ms", (now.elapsed().as_millis()/50));

        let mut buf = GFXBuffer::new(CD24(),1360,768);
        let now = Instant::now();
        for i in 0..50 {
            buf.clear(&BLACK);
        }
        println!("24 bit elapsed {} ms", (now.elapsed().as_millis()/50));
        let mut buf = GFXBuffer::new(CD16(),1360,768);
        let now = Instant::now();
        for i in 0..50 {
            buf.clear(&BLACK);
        }
        println!("16 bit elapsed {} ms", (now.elapsed().as_millis()/50));

    }
}
