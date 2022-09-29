use serde::{Deserialize, Serialize};
use std::fs::{read_to_string};
use std::io::Error;
use crate::graphics::{ARGBColor, GFXBuffer, Size};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlyphInfo {
    pub id:u32,
    pub name:String,
    pub width:i32,
    pub height:i32,
    pub baseline:i32,
    pub data:Vec<u32>,
    pub ascent:i32,
    pub descent:i32,
    pub left:i32,
    pub right:i32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FontInfo2 {
    pub name:String,
    pub glyphs:Vec<GlyphInfo>,
}

impl FontInfo2 {
    pub fn draw_glyph_at(&self, buf: &mut GFXBuffer, ch:u32, x: i32, y: i32, color: &ARGBColor) {
        let mut dx:u32 = x as u32;
        let mut dy:u32 = y as u32;
        for glyph in &self.glyphs {
            if glyph.id == ch {
                for j in 0 .. glyph.height {
                    for i in  glyph.left .. (glyph.width - glyph.right) {
                        let src_n = j * glyph.width + i;
                        let src_bit = glyph.data[src_n as usize];
                        let fx = dx + (i as u32);
                        let fy = dy + (j as u32);
                        if src_bit == 1 {
                            buf.set_pixel_vec_argb(fx as i32, fy as i32, &color.to_argb_vec());
                        }
                    }
                }
                dx += (glyph.width - glyph.left - glyph.right + 1) as u32;
            }
        }
    }
    pub fn draw_text_at(&self, buf: &mut GFXBuffer, text: &str, x: i32, y: i32, color: &ARGBColor) {
        let mut dx:u32 = x as u32;
        let mut dy:u32 = y as u32;
        for ch in text.chars() {
            for glyph in &self.glyphs {
                if glyph.id as u8 as char == ch {
                    for j in 0 .. glyph.height {
                        for i in  glyph.left .. (glyph.width - glyph.right) {
                            let src_n = j * glyph.width + i;
                            let src_bit = glyph.data[src_n as usize];
                            let fx = dx + (i as u32);
                            let fy = dy + (j as u32);
                            if src_bit == 1 {
                                buf.set_pixel_vec_argb(fx as i32, fy as i32, &color.to_argb_vec());
                            }
                        }
                    }
                    dx += (glyph.width - glyph.left - glyph.right + 1) as u32;
                }
            }
        }
    }
    pub fn measure_text(&self, text:&str) -> Size {
        let mut x:i32 = 0;
        let mut h:i32 = 0;
        for ch in text.chars() {
            for glyph in &self.glyphs {
                if glyph.id as u8 as char == ch {
                    x += (glyph.width - glyph.left - glyph.right + 1) as i32;
                    h = i32::max(h,glyph.height);
                }
            }
        }
        Size::init(x+4,h+1)
    }
}

// Result<serde_json::Value, Box<dyn Error>>
pub fn load_font_from_json(json_path: &str) -> Result<FontInfo2, Error> {
    let txt:String = read_to_string(json_path)?;
    let font:FontInfo2 = serde_json::from_str(txt.as_str())?;
    Ok(font)
}
