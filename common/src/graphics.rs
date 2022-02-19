/*
- make screen and graphics abstraction with separate testing and linux impulse
	- Build the rect abstraction l. Use it for drawing rects
	    and window bounds and deciding which
	    window gets the mouse events.
	- create a virtual surface used for testing. then the
	    linux-wm can have multiple gfx implementations.
	    common screen and surface impls.
 */

use fs::canonicalize;
use std::fmt::{format, Formatter, Pointer, Write};
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use log::info;
use png;
use uuid::Uuid;
use crate::{ARGBColor, Point, Rect};
use crate::graphics::ColorDepth::{CD24, CD32};
use crate::graphics::PixelLayout::RGBA;
use serde::{Deserialize, Serialize};
use crate::PixelLayout::ARGB;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ColorDepth {
    CD16(),
    CD24(),
    CD32(),
}

impl ColorDepth {
    pub(crate) fn bytes_per_pixel(&self) -> i32 {
        match self {
            ColorDepth::CD16() => 2,
            ColorDepth::CD24() => 3,
            ColorDepth::CD32() => 4,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum PixelLayout {
    RGB565(),
    RGB(),
    RGBA(),
    ARGB(),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GFXBuffer {
    pub bitdepth:ColorDepth,
    pub layout:PixelLayout,
    pub id:Uuid,
    pub width:u32,
    pub height:u32,
    pub data: Vec<u8>,
    pub fast:bool,
}

impl GFXBuffer {
    pub fn subrect(&self, rect: Rect) -> GFXBuffer {
        let mut sub = GFXBuffer::new(self.bitdepth.clone(),
                                 rect.w as u32,
                                 rect.h as u32,
                                 self.layout.clone());

        for j in 0 .. sub.height {
            for i in 0 .. sub.width {
                let x = rect.x + i as i32;
                if x >= (self.width as i32) { continue; }
                let y = rect.y + j as i32;
                if y >= (self.height as i32) { continue; }
                let val = self.get_pixel_vec_argb(x as u32,y as u32);
                sub.set_pixel_vec_argb(i,j, &val);
            }
        }
        sub
    }
}

impl GFXBuffer {
    pub fn fill_rect_with_image(&mut self, rect: &Rect, buf: &GFXBuffer) {
        // info!("filling rect with image {:?}",rect);
        for j in rect.y .. rect.y + rect.h {
            for i in rect.x .. rect.x + rect.w {
                let v = buf.get_pixel_vec_argb(
                    ( (i - rect.x) as u32 % buf.width) as u32,
                    ( (j - rect.y) as u32 % buf.height) as u32);
                self.set_pixel_vec_argb(i as u32,j as u32,&v);
            }
        }
    }
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
        let mut gfx = GFXBuffer::new(CD32(), info.width, info.height, PixelLayout::ARGB());
        for j in 0..info.height {
            for i in 0..info.width {
                let n = (i + j*info.width) as usize;
                gfx.set_pixel_argb(i,j, bytes[n*4+3],bytes[n*4+0],bytes[n*4+1],bytes[n*4+2]);
            }
        }
        return gfx
    }
}
impl std::fmt::Display for GFXBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("GFXBuffer {:?}{:?} {}x{}",self.bitdepth, self.layout, self.width, self.height).as_str())
    }
}
impl GFXBuffer {
    pub fn draw_image(&mut self, dst_pos:&Point, src_bounds:&Rect, src_buf:&GFXBuffer ) {
        println!("drawing {} to {}  at {}",src_buf,self, dst_pos);
        let mut src_bounds = src_bounds.intersect(self.bounds());
        if src_bounds.is_empty() {
            return;
        }

        let stride = ((self.width as i32) * self.bitdepth.bytes_per_pixel()) as usize;
        for (j, full_dst_row) in self.data.chunks_exact_mut(stride).enumerate() {
            let j = j as i32;
            if j < dst_pos.y {
                continue;
            }
            if j >= dst_pos.y + src_bounds.h {
                continue;
            }
            let src_row_start = (src_buf.bitdepth.bytes_per_pixel() * (src_bounds.x + src_bounds.y * (src_buf.width as i32))) as usize;
            let src_row_len = (src_buf.bitdepth.bytes_per_pixel() * src_bounds.w) as usize;
            let (_, src_row_first) = src_buf.data.split_at(src_row_start);
            let (src_row, _) = src_row_first.split_at(src_row_len);

            let dst_row_start = (self.bitdepth.bytes_per_pixel() * (dst_pos.x + dst_pos.y* (self.width as i32))) as usize;
            let dst_row_len = (self.bitdepth.bytes_per_pixel() * src_bounds.w) as usize;
            let (_, dst_row_first) = full_dst_row.split_at_mut(dst_row_start);
            let (dst_row,_) = dst_row_first.split_at_mut(dst_row_len);
            if self.layout == src_buf.layout {
                // println!("same layout");
                dst_row.copy_from_slice(src_row);
            } else {
                println!("different layout");
            }
        }

        self.copy_from(dst_pos.x, dst_pos.y, src_buf);
    }

    fn copy_from(&mut self, x:i32, y:i32, src: &GFXBuffer) {
        for i in 0..src.width {
            for j in 0..src.height {
                let v = src.get_pixel_vec_argb(i,j);
                self.set_pixel_vec_argb(i+(x as u32), j+(y as u32), &v);
            }
        }
    }

    pub fn fill_rect(&mut self, bounds: Rect, color: &ARGBColor) {
        let bounds = bounds.intersect(self.bounds());
        println!("filling rect {}",bounds);
        let cv = color.as_layout(&self.layout);
        let cv2 = create_filled_row(bounds.w as usize, &cv);
        let bpp:i32 = self.bitdepth.bytes_per_pixel();
        // println!("rect w = {} with len {} row len {}",bounds.w, cv2.len(), row_len);
        for (j,row) in self.data.chunks_exact_mut((self.width as i32 * bpp) as usize).enumerate() {
            let j = j as i32;
            if j < bounds.x {
                continue;
            }
            if j >= bounds.y + bounds.h {
                continue;
            }
            let (_, after) = row.split_at_mut((bounds.x * bpp) as usize);
            let (mut middle, _) = after.split_at_mut((bounds.w * bpp) as usize);
            middle.copy_from_slice(&cv2);
        }
    }
    pub fn draw_rect(&mut self, bounds: Rect, color: &ARGBColor, size: i32) {
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
            println!("get error. pixel {},{} out of bounds {}x{}",x,y,self.width,self.height);
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
                v[0] = 255;
                v[1] = self.data[n*3+0];
                v[2] = self.data[n*3+1];
                v[3] = self.data[n*3+2];
            }
            ColorDepth::CD32() => {
                let n = (x + y * (self.width as u32)) as usize;
                match self.layout {
                    PixelLayout::ARGB() => {
                        v[0] = self.data[n*4+0];
                        v[1] = self.data[n*4+1];
                        v[2] = self.data[n*4+2];
                        v[3] = self.data[n*4+3];
                    }
                    PixelLayout::RGBA() => {
                        v[0] = self.data[n*4+3];
                        v[1] = self.data[n*4+0];
                        v[2] = self.data[n*4+1];
                        v[3] = self.data[n*4+2];
                    }
                    _ => {}
                }
            }
        }
        return v
    }
    pub fn get_pixel_vec(&self, layout: &PixelLayout, x: i32, y: i32) -> Vec<u8> {
        let pix = self.get_pixel_vec_argb(x as u32, y as u32);
        println!("pix is {:?}",pix);
        let color = ARGBColor::from_argb_vec(&pix);
        return color.as_layout(&layout);
    }

    pub fn set_pixel_vec_argb(&mut self, x:u32, y:u32, v:&Vec<u8>) {
        if x >= self.width || y >= self.height {
            println!("set error. pixel {},{} out of bounds {}x{}",x,y,self.width,self.height);
            return;
        }
        match self.bitdepth {
            ColorDepth::CD16() => {
                match self.layout {
                    PixelLayout::RGB565() => {
                        let n = (x + y * (self.width as u32)) as usize;
                        let r = v[1];
                        let g = v[2];
                        let b = v[3];
                        let upper = ((r >> 3)<<3) | ((g & 0b111_00000) >> 5);
                        let lower = (((g & 0b00011100) >> 2) << 5) | ((b & 0b1111_1000) >> 3);
                        self.data[n*2+0] = lower;
                        self.data[n*2+1] = upper;
                    }
                    _ => {}
                }
            }
            ColorDepth::CD24() => {
                let n = (x + y * (self.width as u32)) as usize;
                self.data[n*3+0] = v[1];
                self.data[n*3+1] = v[2];
                self.data[n*3+2] = v[3];
            }
            ColorDepth::CD32() => {
                let n = (x + y * (self.width as u32)) as usize;
                match self.layout {
                    PixelLayout::ARGB() => {
                        self.data[n*4+0] = v[0];
                        self.data[n*4+1] = v[1];
                        self.data[n*4+2] = v[2];
                        self.data[n*4+3] = v[3];
                    }
                    PixelLayout::RGBA() => {
                        self.data[n*4+0] = v[3];
                        self.data[n*4+1] = v[2];
                        self.data[n*4+2] = v[1];
                        self.data[n*4+3] = v[0];
                    }
                    _ => {}
                }
            }
        }
    }
    pub fn set_pixel_argb(&mut self, x:u32, y:u32, a:u8,r:u8,g:u8,b:u8) {
        self.set_pixel_vec_argb(x,y,&vec![a,r,g,b]);
    }
    pub fn bounds(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: self.width as i32,
            h: self.height as i32,
        }
    }
}

impl GFXBuffer {
    pub fn clear(&mut self, color: &ARGBColor) {
        //println!("clearing {:?} {:?}  {}x{}",self.bitdepth, self.layout, self.width, self.height);
        match self.bitdepth {
            ColorDepth::CD16() => {
                match self.layout {
                    PixelLayout::RGB565() => {
                        if self.fast {
                            let cv = color.as_layout(&self.layout);
                            let cv2 = create_filled_row(self.width as usize, &cv);
                            for chunk in self.data.chunks_exact_mut(cv2.len()) {
                                chunk.copy_from_slice(&cv2);
                            }
                        } else {
                            let v = color.to_argb_vec();
                            let r = v[1];
                            let g = v[2];
                            let b = v[3];
                            let upper = ((r >> 3) << 3) | ((g & 0b111_00000) >> 5);
                            let lower = (((g & 0b00011100) >> 2) << 5) | ((b & 0b1111_1000) >> 3);
                            let mut row: Vec<u8> = vec![];
                            for i in 0..self.width {
                                row.push(lower);
                                row.push(upper);
                            }
                            for chunk in self.data.chunks_exact_mut((self.width * 2) as usize) {
                                chunk.copy_from_slice(&*row);
                            }
                        }
                    }
                    _ => {}
                }
            }
            CD24() => {
                if self.fast {
                    // println!("pixel layout is {:?}", self.layout);
                    let cv = color.as_layout(&self.layout);
                    let cv2 = create_filled_row(self.width as usize, &cv);
                    for chunk in self.data.chunks_exact_mut(cv2.len()) {
                        chunk.copy_from_slice(&cv2);
                    }
                } else {
                    self.fill_rect(Rect::from_ints(0, 0, self.width as i32, self.height as i32), color);
                }
            }
            CD32() => {
                if self.fast {
                    // println!("pixel layout is {:?}", self.layout);
                    let cv = color.as_layout(&self.layout);
                    let cv2 = create_filled_row(self.width as usize, &cv);
                    for chunk in self.data.chunks_exact_mut(cv2.len()) {
                        chunk.copy_from_slice(&cv2);
                    }
                } else {
                    self.fill_rect(Rect::from_ints(0, 0, self.width as i32, self.height as i32), color);
                }
            }
        }
    }
}

fn create_filled_row(size: usize, color: &Vec<u8>) -> Vec<u8> {
    let mut nv:Vec<u8> = vec![0;size*color.len()];
    for n in nv.chunks_exact_mut(color.len()) {
        n.copy_from_slice(color);
    }
    return nv;
}

impl GFXBuffer {
    pub fn new(bitdepth:ColorDepth, width:u32, height:u32, layout: PixelLayout) -> GFXBuffer {
        if width == 0 || height == 0 {
            panic!("cannot create buffer of size {}x{}",width,height);
        }
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
            layout,
            fast: false
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
    use std::path::{Path, PathBuf};
    use std::time::Instant;
    use crate::{ARGBColor, BLACK, Rect, WHITE};
    use crate::font::load_font_from_json;
    use crate::graphics::ColorDepth::{CD16, CD24, CD32};
    use crate::graphics::{draw_test_pattern, export_to_png, GFXBuffer, PixelLayout};
    use crate::graphics::PixelLayout::ARGB;

    // check ARGB memory buffer copied to RGBA() framebuffer
    // check color drawn to RGBA() framebuffer
    // check ARGB memory buffer copied to RGB565() framebuffer
    #[test]
    fn ARGB_to_RGBA() {
        let RED = ARGBColor::new_rgb(255,0,0);
        let GREEN = ARGBColor::new_rgb(0,255,0);
        let BLUE = ARGBColor::new_rgb(0,0,255);
        // let ARGB_RED = vec![255,255,0,0];

        //=== RGBA ===
        let mut buf2 = GFXBuffer::new(CD32(), 1, 1, PixelLayout::RGBA());
        buf2.clear(&RED);
        assert_eq!(buf2.data,vec![255,0,0,255]);
        buf2.clear(&GREEN);
        assert_eq!(buf2.data,vec![0,255,0,255]);
        buf2.fill_rect(Rect::from_ints(0,0,1,1), &RED);
        assert_eq!(buf2.data,vec![255,0,0,255]);
        buf2.fill_rect(Rect::from_ints(0,0,1,1), &GREEN);
        assert_eq!(buf2.data,vec![0,255,0,255]);

        //=== ARGB ===
        let mut buf1 = GFXBuffer::new(CD32(), 1, 1, PixelLayout::ARGB());
        buf1.clear(&RED);
        assert_eq!(buf1.data,vec![255,255,0,0]);
        buf1.clear(&GREEN);
        assert_eq!(buf1.data,vec![255,0,255,0]);


        //copy ARGB to RGBA
        buf1.clear(&BLUE);
        assert_eq!(buf1.data,vec![255,0,0,255]);
        buf2.copy_from(0,0,&buf1);
        assert_eq!(buf2.data,vec![0,0,255,255]);

        //copy RGBA to ARGB
        buf2.clear(&BLUE);
        assert_eq!(buf2.data,vec![0,0,255,255]);
        buf1.copy_from(0,0,&buf2);
        assert_eq!(buf1.data,vec![255,0,0,255]);
    }

    #[test]
    fn ARGB_to_RGB565() {
        let RED = ARGBColor::new_rgb(255,0,0);
        let GREEN = ARGBColor::new_rgb(0,255,0);
        let BLUE = ARGBColor::new_rgb(0,0,255);

        //=== RGB565 ===
        let mut buf2 = GFXBuffer::new(CD16(), 1, 1, PixelLayout::RGB565());
        buf2.clear(&RED);
        assert_eq!(buf2.data,vec![0b000_00000,0b11111_000,]);
        buf2.clear(&GREEN);
        assert_eq!(buf2.data,vec![0b111_00000,0b00000_111,]);
        buf2.fill_rect(Rect::from_ints(0,0,1,1), &RED);
        assert_eq!(buf2.data,vec![0b000_00000,0b11111_000,]);
        buf2.fill_rect(Rect::from_ints(0,0,1,1), &GREEN);
        assert_eq!(buf2.data,vec![0b111_00000,0b00000_111,]);

        // === copy ARGB to RGB565
        let mut buf1 = GFXBuffer::new(CD32(), 1, 1, PixelLayout::ARGB());
        buf1.clear(&BLUE);
        assert_eq!(buf1.data,vec![255,0,0,255]);
        buf2.copy_from(0,0,&buf1);
        assert_eq!(buf2.data,vec![0b000_11111,0b00000_000, ]);

        //copy RGB565 to ARGB
        buf2.clear(&BLUE);
        assert_eq!(buf2.data,vec![ 0b000_11111, 0b00000_000,]);
        buf1.copy_from(0,0,&buf2);
        assert_eq!(buf1.data,vec![255,24,224,0]);
    }

    /*
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
            let mut buf = GFXBuffer::new(CD24(), 2, 2, );
            buf.clear(&set_color);
            let color = buf.get_pixel_vec_argb(0, 0);
            assert_eq!(color,set_color.as_vec());
        }
        for set_color in &colors {
            let mut buf = GFXBuffer::new(CD32(), 2, 2, );
            buf.clear(&set_color);
            let color = buf.get_pixel_vec_argb(0, 0);
            assert_eq!(color,set_color.as_vec());
        }
        // println!("color is {:?}",color);
        // println!("data is {:?}",buf.data);
    }

    #[test]
    fn check_24_to_16() {
        let mut buf24 = GFXBuffer::new(CD24(), 2, 2, );
        let green = ARGBColor::new_rgb(0, 255, 0);
        buf24.clear(&green);
        let mut buf16 = GFXBuffer::new(CD16(), 2, 2, );
        buf16.copy_from(0,0,&buf24);
        {
            let c1 = buf16.get_pixel_vec_argb(1, 1);
            assert_eq!(c1, vec![255,0,248,0]);//0b11111111_00000000_11111100_00000000);
        }
    }

    #[test]
    fn check_32_to_32() {
        let mut buf = GFXBuffer::new(CD32(), 2, 2, );
        buf.set_pixel_argb(0,0, 255,255,254,253);
        // buf.set_pixel_32argb(0,0, ARGBColor::new_rgb(255,254,253).as_32bit());
        // print!("{:x} vs {:x}", ARGBColor::new_rgb(255,254,253).as_32bit(), buf.get_pixel_32argb(0,0));
        assert_eq!(buf.get_pixel_vec_argb(0,0),vec![255,255,254,253]);
        // assert_eq!(buf.get_pixel_32argb(0,0),0xFFFFFEFD);
    }

    #[test]
    fn try_test_pattern() {
        let mut buf = GFXBuffer::new(CD32(), 64, 64, );
        draw_test_pattern(&mut buf);

        // buf = GFXBuffer::new(CD32(),32,32);
        // draw_test_pattern(&mut buf);
        export_to_png(&buf);
    }


    #[test]
    fn drawing_speed() {
        let mut buf = GFXBuffer::new(CD32(), 1360, 768, );
        let now = Instant::now();
        for i in 0..50 {
            buf.clear(&BLACK);
        }
        println!("32bit elapsed {} ms", (now.elapsed().as_millis()/50));

        let mut buf = GFXBuffer::new(CD24(), 1360, 768, );
        let now = Instant::now();
        for i in 0..50 {
            buf.clear(&BLACK);
        }
        println!("24 bit elapsed {} ms", (now.elapsed().as_millis()/50));
        let mut buf = GFXBuffer::new(CD16(), 1360, 768, );
        let now = Instant::now();
        for i in 0..50 {
            buf.clear(&BLACK);
        }
        println!("16 bit elapsed {} ms", (now.elapsed().as_millis()/50));

    }
    */

    #[test]
    fn test_font_load() {
        println!("current dir = {:?}",std::env::current_dir());
        let font = load_font_from_json("../resources/default-font.json").unwrap();
    }

    #[test]
    fn draw_bitmap() {
        let mut bitmap = GFXBuffer::new(CD32(), 200, 100, PixelLayout::RGBA());
        bitmap.clear(&WHITE);
        let font = load_font_from_json("../resources/default-font.json").unwrap();
        font.draw_text_at(&mut bitmap, "Greetings, Earthling!", 20, 20, &ARGBColor::new_argb(255,0, 255, 0));
        export_to_png(&bitmap, &PathBuf::from("earthling.png"));
    }
}

pub fn export_to_png(buf: &GFXBuffer, pth:&PathBuf) {
    let file = File::create(pth).unwrap();
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
    println!("exported to {:?}", fs::canonicalize(&pth).unwrap());
}
