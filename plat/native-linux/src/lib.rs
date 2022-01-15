use std::sync::mpsc::Sender;
use common::{ARGBColor, IncomingMessage, Rect};
use common::graphics::GFXBuffer;

pub struct Plat {

}

pub fn make_plat<'a>(sender: Sender<IncomingMessage>) -> Result<Plat, String> {
    return Ok(Plat {
        // sender: sender,
    });
}

impl Plat {
    pub fn get_screen_bounds(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0
        }
    }
    pub fn service_input(&mut self) {}
    pub fn service_loop(&mut self) {}
    pub fn fill_rect(&mut self, rect: Rect, color: &ARGBColor) {}
    pub fn draw_rect(&mut self, rect: Rect, color: &ARGBColor, width: i32) {}
    pub fn draw_image(&mut self, x: i32, y: i32, img: &GFXBuffer) {}
    pub fn clear(&mut self) {}
    pub fn shutdown(&mut self) {}
    pub fn register_image2(&mut self) {

    }
}

