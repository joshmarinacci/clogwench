use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::{PathBuf};
use log::{error, warn};
use png;
use uuid::Uuid;
use crate::{ARGBColor, Point, Rect};
use serde::{Deserialize, Serialize};
use crate::PixelLayout::{ARGB, RGB565};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum PixelLayout {
    RGB565(),
    // RGB(),
    // RGBA(),
    ARGB(),
}
impl PixelLayout {
    pub(crate) fn bytes_per_pixel(&self) -> i32 {
        match self {
            PixelLayout::RGB565() => 2,
            // PixelLayout::RGB() => 3,
            // PixelLayout::RGBA() => 4,
            PixelLayout::ARGB() => 4,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GFXBuffer {
    pub layout:PixelLayout,
    pub id:Uuid,
    pub width:u32,
    pub height:u32,
    pub data: Vec<u8>,
}

impl GFXBuffer {
    pub fn new(width:u32, height:u32, layout: &PixelLayout) -> GFXBuffer {
        if width <= 0 || height <= 0 {
            panic!("cannot create buffer of size {}x{}",width,height);
        }
        let byte_length = (layout.bytes_per_pixel() as u32) * width * height;
        let data = vec![0u8; byte_length as usize];
        GFXBuffer {
            data,
            width,
            height,
            id: Uuid::new_v4(),
            layout:layout.clone(),
        }
    }
    pub fn from_png_file(path: &str) -> GFXBuffer {
        let decoder = png::Decoder::new(File::open(path).unwrap());
        let mut reader = decoder.read_info().unwrap();
        println!("loading bytes {}", reader.output_buffer_size());
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        println!("size {}x{} bit depth={:?} color type={:?}",info.width, info.height, info.bit_depth, info.color_type);
        let bytes = &buf[..info.buffer_size()];
        let mut gfx = GFXBuffer::new(info.width, info.height, &PixelLayout::ARGB());
        for j in 0..info.height {
            for i in 0..info.width {
                let n = (i + j*info.width) as usize;
                gfx.set_pixel_argb(i as i32, j as i32, bytes[n*4+3], bytes[n*4+0], bytes[n*4+1], bytes[n*4+2]);
            }
        }
        return gfx
    }
}
impl std::fmt::Display for GFXBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("GFXBuffer {:?} {}x{}", self.layout, self.width, self.height).as_str())
    }
}


impl GFXBuffer {
    pub fn to_layout(&self, layout: &PixelLayout) -> GFXBuffer {
        let mut buf = GFXBuffer::new(self.width, self.height, layout);
        for j in 0 .. self.height {
            for i in 0.. self.width {
                let v = self.get_pixel_vec_argb(i as i32, j as i32);
                buf.set_pixel_vec_argb(i as i32, j as i32, &v);
            }
        }
        return buf;
    }
    pub fn sub_rect(&self, rect: Rect) -> GFXBuffer {
        let mut sub = GFXBuffer::new(
                                 rect.w as u32,
                                 rect.h as u32,
                                 &self.layout);

        for j in 0 .. sub.height {
            for i in 0 .. sub.width {
                let x = rect.x + i as i32;
                if x >= (self.width as i32) { continue; }
                let y = rect.y + j as i32;
                if y >= (self.height as i32) { continue; }
                let val = self.get_pixel_vec_argb(x as i32,y as i32);
                sub.set_pixel_vec_argb(i as i32, j as i32, &val);
            }
        }
        sub
    }
    pub fn fill_rect_with_image(&mut self, rect: &Rect, buf: &GFXBuffer) {
        // info!("filling rect with image {:?}",rect);
        for j in rect.y .. rect.y + rect.h {
            for i in rect.x .. rect.x + rect.w {
                let v = buf.get_pixel_vec_argb(
                    ( (i - rect.x) as u32 % buf.width) as i32,
                    ( (j - rect.y) as u32 % buf.height) as i32);
                self.set_pixel_vec_argb(i as i32,j as i32,&v);
            }
        }
    }
    pub fn draw_image(&mut self, dst_pos:&Point, src_bounds:&Rect, src_buf:&GFXBuffer ) {
        let mut dst_f_bounds = src_bounds.add(dst_pos).intersect(self.bounds());
        if dst_f_bounds.is_empty() { return; }
        let self_bounds = self.bounds().clone();
        let src_f_bounds = dst_f_bounds.subtract(dst_pos);
        // println!("drawing {} to {}  at {}  with {} {}", src_buf, self, dst_pos, src_bounds, self_bounds);
        // println!("drawing in src {}",src_f_bounds);
        // println!("drawing in dst {}", dst_f_bounds);
        if src_buf.layout == self.layout {
            //println!("same layout");
            let dst_x = dst_f_bounds.x;
            let src_x = src_f_bounds.x;
            let src_bbp = src_buf.layout.bytes_per_pixel();
            let dst_bpp = self.layout.bytes_per_pixel();
            let src_w = src_buf.width as i32;
            let dst_w = self.width as i32;

            for dst_y in dst_f_bounds.y .. dst_f_bounds.y + dst_f_bounds.h {
                let row_len = src_f_bounds.w as i32;
                let src_y = dst_y - dst_f_bounds.y + src_f_bounds.y;
                let src_row_start = (src_bbp * (src_y * src_w + src_x)) as usize;
                let dst_row_start = (dst_bpp * (dst_y * dst_w + dst_x)) as usize;
                let src_row_len   = (src_bbp * row_len) as usize;
                let dst_row_len   = (dst_bpp * row_len) as usize;

                let src_data = &src_buf.data[src_row_start .. src_row_start + src_row_len];
                let dst_data = &mut self.data[dst_row_start .. dst_row_start + dst_row_len];
                dst_data.copy_from_slice(src_data);
            }
        } else {
            println!("different layout");
            for j in dst_f_bounds.y .. dst_f_bounds.y + dst_f_bounds.h {
                for i in dst_f_bounds.x .. dst_f_bounds.x + dst_f_bounds.w {
                    let v = src_buf.get_pixel_vec_argb(i, j);
                    self.set_pixel_vec_argb(i+dst_pos.x, j+dst_pos.y, &v);
                }
            }
        }
    }
    fn stride(&self) -> usize {
        ((self.layout.bytes_per_pixel() as u32) * self.width) as usize
    }
}

impl GFXBuffer {
    pub fn clear(&mut self, color: &ARGBColor) {
        //println!("clearing {:?} {:?}  {}x{}",self.bit_depth, self.layout, self.width, self.height);
        match self.layout {
            RGB565() => {
                let cv = color.as_layout(&self.layout);
                let cv2 = create_filled_row(self.width as usize, &cv);
                for chunk in self.data.chunks_exact_mut(cv2.len()) {
                    chunk.copy_from_slice(&cv2);
                }
            }
            ARGB() => {
                // println!("pixel layout is {:?}", self.layout);
                let cv = color.as_layout(&self.layout);
                let cv2 = create_filled_row(self.width as usize, &cv);
                for chunk in self.data.chunks_exact_mut(cv2.len()) {
                    chunk.copy_from_slice(&cv2);
                }
            }
        }
    }
    pub fn fill_rect(&mut self, bounds: Rect, color: &ARGBColor) {
        let bounds = bounds.intersect(self.bounds());
        let cv = color.as_layout(&self.layout);
        let cv2 = create_filled_row(bounds.w as usize, &cv);
        let bpp:i32 = self.layout.bytes_per_pixel();
        for (j,row) in self.data.chunks_exact_mut((self.width as i32 * bpp) as usize).enumerate() {
            let j = j as i32;
            if j < bounds.y {
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
    pub fn get_pixel_vec_argb(&self, x:i32, y:i32) -> Vec<u8>{
        let mut v:Vec<u8> = vec![0,0,0,0];
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
            println!("get error. pixel {},{} out of bounds {}x{}",x,y,self.width,self.height);
            return v;
        }
        match self.layout {
            PixelLayout::RGB565() => {
                let n = (x + y * self.width as i32) as usize;
                let lower = self.data[n*2+0];
                let upper = self.data[n*2+1];
                //red = up[7-3]
                //gre = up[2-0] | low[7-5]
                //blu = up[4-0]
                // let packed_color:u16 = ((self.data[n*2+0] as u16) << 8) | (self.data[n*2+1] as u16);
                // return ARGBColor::from_16bit(packed_color).to_argb_vec();
                let r:u8 = (upper & 0b11111_000);
                let g:u8 = ((upper & 0b0000_0111) << 5) | ((lower & 0b1110_0000) >> 5);
                let b:u8 = (lower & 0b0001_1111)  << 3;
                v[0] = 255;
                v[1] = r;
                v[2] = g;
                v[3] = b;
            }
            PixelLayout::ARGB() => {
                let n = (x + y * (self.width as i32)) as usize;
                v[0] = self.data[n*4+0];
                v[1] = self.data[n*4+1];
                v[2] = self.data[n*4+2];
                v[3] = self.data[n*4+3];
            }
        }
        return v
    }
    pub fn get_pixel_vec_as_layout(&self, layout: &PixelLayout, x: i32, y: i32) -> Vec<u8> {
        let pix = self.get_pixel_vec_argb(x , y);
        let color = ARGBColor::from_argb_vec(&pix);
        return color.as_layout(&layout);
    }
    pub fn set_pixel_vec_argb(&mut self, x:i32, y:i32, v:&Vec<u8>) {
        if x >= self.width as i32 || y >= self.height as i32 {
            println!("set error. pixel {},{} out of bounds {}x{}",x,y,self.width,self.height);
            return;
        }
        match self.layout {
            PixelLayout::RGB565() => {
                let n = (x + y * (self.width as i32)) as usize;
                let r = v[1];
                let g = v[2];
                let b = v[3];
                let upper = ((r >> 3) << 3) | ((g & 0b111_00000) >> 5);
                let lower = (((g & 0b00011100) >> 2) << 5) | ((b & 0b1111_1000) >> 3);
                self.data[n * 2 + 0] = lower;
                self.data[n * 2 + 1] = upper;
            }
            PixelLayout::ARGB() => {
                let n = (x + y * (self.width as i32)) as usize;
                self.data[n*4+0] = v[0];
                self.data[n*4+1] = v[1];
                self.data[n*4+2] = v[2];
                self.data[n*4+3] = v[3];
            }
        }
    }
    pub fn set_pixel_argb(&mut self, x:i32, y:i32, a:u8,r:u8,g:u8,b:u8) {
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
    pub fn to_png(&self, pth:&PathBuf) {
        let file = File::create(pth).unwrap();
        let ref mut w = BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, self.width, self.height); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();

        let mut data:Vec<u8> = vec![];
        for j in 0..self.height {
            for i in 0..self.width {
                let px = self.get_pixel_vec_argb(i as i32, j as i32);
                // println!("{},{}  {:x}",i,j, buf.get_pixel_32argb(i,j));
                data.push(px[1]); //R
                data.push(px[2]); //G
                data.push(px[3]); //B
                data.push(px[0]); //A
            }
        }
        writer.write_image_data(&data).unwrap(); // Save
        println!("exported to {:?}", fs::canonicalize(&pth).unwrap());
    }
}

fn create_filled_row(size: usize, color: &Vec<u8>) -> Vec<u8> {
    let mut nv:Vec<u8> = vec![0;size*color.len()];
    for n in nv.chunks_exact_mut(color.len()) {
        n.copy_from_slice(color);
    }
    return nv;
}


pub fn draw_test_pattern(buf:&mut GFXBuffer) {
    for j in 0..buf.height {
        for i in 0..buf.width {
            let v = (i*4) as u8;
            if j == 0 || j == (buf.height-1) {
                buf.set_pixel_argb(i as i32, j as i32, 255, 255, 255, 255);
                continue;
            }
            if j < (buf.height/4)*1 {
                buf.set_pixel_argb(i as i32, j as i32, 255, v, 0, 0);
                continue;
            }
            if j < (buf.height/4)*2 {
                buf.set_pixel_argb(i as i32, j as i32, 255, 0, v, 0);
                continue;
            }
            if j < (buf.height/4)*3 {
                buf.set_pixel_argb(i as i32, j as i32, 255, 0, 0, v);
                continue;
            }
            if j < (buf.height/4)*4 {
                buf.set_pixel_argb(i as i32, j as i32, v, 128, 128, 128);
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{PathBuf};
    use std::time::Instant;
    use crate::{ARGBColor, BLACK, Point, Rect, WHITE};
    use crate::font::load_font_from_json;
    use crate::graphics::{draw_test_pattern, GFXBuffer};
    use crate::graphics::PixelLayout;


    // check ARGB memory buffer copied to RGBA() framebuffer
    // check color drawn to RGBA() framebuffer
    // check ARGB memory buffer copied to RGB565() framebuffer
    #[test]
    fn argb_clear_fill() {
        let red = ARGBColor::new_rgb(255, 0, 0);
        let green = ARGBColor::new_rgb(0, 255, 0);
        let blue = ARGBColor::new_rgb(0, 0, 255);

        //=== ARGB ===
        let mut buf2 = GFXBuffer::new(1, 1, &PixelLayout::ARGB());
        buf2.clear(&red);
        assert_eq!(buf2.data,vec![255, 255,0,0]);
        buf2.clear(&green);
        assert_eq!(buf2.data,vec![255,0,255,0]);
        buf2.clear(&blue);
        assert_eq!(buf2.data,vec![255,0,0,255]);
        buf2.fill_rect(Rect::from_ints(0,0,1,1), &red);
        assert_eq!(buf2.data,vec![255,255,0,0]);
        buf2.fill_rect(Rect::from_ints(0,0,1,1), &green);
        assert_eq!(buf2.data,vec![255,0,255,0]);
        buf2.fill_rect(Rect::from_ints(0,0,1,1), &blue);
        assert_eq!(buf2.data,vec![255,0,0,255]);
    }

    #[test]
    fn argb_to_rgb565() {
        let red = ARGBColor::new_rgb(255, 0, 0);
        let grn = ARGBColor::new_rgb(0, 255, 0);
        let blu = ARGBColor::new_rgb(0, 0, 255);

        //=== RGB565 ===
        let mut buf2 = GFXBuffer::new(1, 1, &PixelLayout::RGB565());
        buf2.clear(&red);
        assert_eq!(buf2.data,vec![0b000_00000,0b11111_000,]);
        buf2.clear(&grn);
        assert_eq!(buf2.data,vec![0b111_00000,0b00000_111,]);
        buf2.fill_rect(Rect::from_ints(0,0,1,1), &red);
        assert_eq!(buf2.data,vec![0b11111_000,0b000_00000,]);
        buf2.fill_rect(Rect::from_ints(0,0,1,1), &grn);
        assert_eq!(buf2.data,vec![0b00000_111,0b111_00000,]);

        // === copy ARGB to RGB565
        let mut buf1 = GFXBuffer::new(1, 1, &PixelLayout::ARGB());
        buf1.clear(&blu);
        assert_eq!(buf1.data,vec![255,0,0,255]);
        buf2.draw_image(&Point::init(0, 0), &buf1.bounds(), &buf1);
        assert_eq!(buf2.data,vec![0b000_11111,0b00000_000, ]);

        //copy RGB565 to ARGB
        buf2.clear(&blu);
        assert_eq!(buf2.data,vec![ 0b000_11111, 0b00000_000,]);
        buf1.draw_image(&Point::init(0,0),&buf2.bounds(),&buf2);
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
        let _font = load_font_from_json("../resources/default-font.json").unwrap();
    }

    #[test]
    fn draw_bitmap() {
        let mut bitmap = GFXBuffer::new(200, 100, &PixelLayout::ARGB());
        bitmap.clear(&WHITE);
        let font = load_font_from_json("../resources/default-font.json").unwrap();
        font.draw_text_at(&mut bitmap, "Greetings, Earthling!", 20, 20, &ARGBColor::new_argb(255,0, 255, 0));
        bitmap.to_png(&PathBuf::from("earthling.png"));
    }

    #[test]
    fn buffer_clear_cd32_rgba_speed() {
        let start = Instant::now();
        let w = 1024;
        let h = 1024;
        let color = ARGBColor::new_rgb(100,100,100);
        let mut background = GFXBuffer::new( w, h, &PixelLayout::ARGB());
        for _ in 0..10 {
            background.clear(&color);
        }
        // 2.55s vs 0.76s  ~= 3.3x faster
        println!("took {}",start.elapsed().as_secs_f32());
    }
    #[test]
    fn buffer_clear_cd24_rgb_speed() {
        let start = Instant::now();
        let w = 1024;
        let h = 1024;
        let color = ARGBColor::new_rgb(100,100,100);
        let mut background = GFXBuffer::new( w, h, &PixelLayout::ARGB());
        for _ in 0..10 {
            background.clear(&color);
        }
        // 2.55s vs 0.76s  ~= 3.3x faster
        println!("took {}",start.elapsed().as_secs_f32());
    }


    #[test]
    fn buffer_clear_cd15_rgb565_speed() {
        let start = Instant::now();
        let w = 1024;
        let h = 1024;
        let color = ARGBColor::new_rgb(100,100,100);
        let mut background = GFXBuffer::new(w, h, &PixelLayout::RGB565());
        for _ in 0..10 {
            background.clear(&color);
        }
        // 2.55s vs 0.76s  ~= 3.3x faster
        println!("took {}",start.elapsed().as_secs_f32());
    }


    #[test]
    fn buffer_fill_rect_cd32_rgba_speed() {
        let w = 1024;
        let h = 1024;
        let color = ARGBColor::new_rgb(96,100,96);

        let types = [PixelLayout::ARGB(), PixelLayout::RGB565()];

        for layout in types {
            let mut background = GFXBuffer::new(w, h, &layout);
            background.clear(&BLACK);
            let start = Instant::now();
            let bounds = Rect::from_ints(500, 500, 1000, 1000);
            for _ in 0..10 {
                background.fill_rect(bounds, &color);
            }
            println!("took {}",start.elapsed().as_secs_f32());
            assert_eq!(background.get_pixel_vec_as_layout(&PixelLayout::ARGB(), 0, 0), BLACK.as_layout(&PixelLayout::ARGB()));
            assert_eq!(background.get_pixel_vec_as_layout(&PixelLayout::ARGB(), 600, 600), color.as_layout(&PixelLayout::ARGB()));
        }

        // 2.55s vs 0.76s  ~= 3.3x faster

    }

    #[test]
    fn buffer_draw_image_speed() {
        let w = 1024;
        let h = 1024;
        let types = [PixelLayout::ARGB(),PixelLayout::RGB565()];

        let mut src_img = GFXBuffer::new(500, 500, &PixelLayout::ARGB());
        src_img.clear(&BLACK);
        src_img.fill_rect(Rect::from_ints(0,0,250,250),&WHITE);
        src_img.fill_rect(Rect::from_ints(250,250,250,250),&WHITE);
        // export_to_png(&src_img, &PathBuf::from("pattern.png"));
        for layout in &types {
            let mut background = GFXBuffer::new( w, h, layout);
            background.clear(&BLACK);

            for layout2  in &types {
                let src_img = src_img.to_layout(layout2);
                let start = Instant::now();
                for _ in 0..10 {
                    background.draw_image(&Point::init(0, 0),
                                          &src_img.bounds(), &src_img);
                }
                println!("took {} {:?} -> {:?}", start.elapsed().as_secs_f32(), layout2, layout);
                // println!("is black {:?}",background.get_pixel_vec(&PixelLayout::RGBA(),0,0));
                // println!("is color {:?}",background.get_pixel_vec(&PixelLayout::RGBA(),600,600));
                // assert_eq!(background.get_pixel_vec(&PixelLayout::RGBA(), 0, 0), WHITE.as_layout(&PixelLayout::RGBA()));
                // assert_eq!(background.get_pixel_vec(&PixelLayout::RGBA(), 0, 256), BLACK.as_layout(&PixelLayout::RGBA()));
            }
        }
    }

    #[test]
    fn buffer_draw_correctness() {
        let mut src_img = GFXBuffer::new(100, 100, &PixelLayout::ARGB());
        draw_test_pattern(&mut src_img);

        let mut dst_img = GFXBuffer::new(200,200, &PixelLayout::ARGB());
        dst_img.draw_image(&Point::init(150, 150), &src_img.bounds(), &src_img);
    }

    #[test]
    fn buffer_image_conversion_correctness() {
        let mut buf1 = GFXBuffer::new(256, 256, &PixelLayout::ARGB());
        draw_test_pattern(&mut buf1);
        buf1.to_png(&PathBuf::from("test_pattern_1.png"));

        let mut buf2 = buf1.to_layout(&PixelLayout::ARGB());
        buf2.to_png(&PathBuf::from("test_pattern_2.png"));

        let mut buf3 = buf1.to_layout(&PixelLayout::RGB565());
        buf3.to_png(&PathBuf::from("test_pattern_3.png"));


        let mut cursor = GFXBuffer::from_png_file("../resources/cursor.png");
        cursor.to_png(&PathBuf::from("cursor_1.png"));

        cursor.to_layout(&PixelLayout::RGB565()).to_png(&PathBuf::from("cursor_2.png"));
    }

}

