use std::fmt::Formatter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::events::{KeyDownEvent, KeyUpEvent, MouseDownEvent};
use crate::graphics::{GFXBuffer, PixelLayout};

pub mod client;
pub mod events;
pub mod graphics;
pub mod font;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ARGBColor {
    pub r:u8,
    pub g:u8,
    pub b:u8,
    pub a:u8,
}

/*
impl ARGBColor {
    pub(crate) fn from_16bit(packed_color: u16) -> ARGBColor {
        let r:u8 = (((packed_color & 0b11111_000000_00000) >> 11) << 3) as u8;
        let g:u8 = (((packed_color & 0b00000_111111_00000) >> 5)  << 2) as u8;
        let b:u8 = (((packed_color & 0b00000_000000_11111) >> 0)  << 3) as u8;
        return ARGBColor::new_rgb(r, g, b);
    }
    pub(crate) fn from_24bit(packed_color: u32) -> ARGBColor {
        let r:u8 = ((packed_color & 0xFF0000) >> 16) as u8;
        let g:u8 = ((packed_color & 0x00FF00) >> 8) as u8;
        let b:u8 = ((packed_color & 0x0000FF) >> 0) as u8;
        return ARGBColor::new_rgb(r, g, b);
    }
}
*/
/*
impl ARGBColor {
    pub fn as_16bit(&self) -> u16 {
        // println!("color {:?}",self);
        let r = self.r >> 3; // 5 bits
        let g = self.g >> 2; // 6 bits
        let b = self.b >> 3; // 5 bits
        // println!("parts are {} {} {}",r,g,b);
        return (((r as u16) << (5+6)) | ((g as u16) << 5) | ((b as u16) << 0)) as u16;
    }
    pub fn as_24bit(&self) -> u32 {
        return ((self.r as u32) << 16) | ((self.g as u32) << 8) | ((self.b as u32) << 0) as u32;
    }
    pub fn as_32bit(&self) -> u32 {
        // println!("parts are")
        return ((self.a as u32)<<24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | ((self.b as u32) << 0) as u32;
    }
}
*/

impl ARGBColor {
    // pub(crate) fn as_vec(&self) -> Vec<u8> {
    //     Vec::from([self.a, self.r, self.g, self.b])
    // }
    pub fn to_argb_vec(&self) -> Vec<u8> {
        vec![self.a,self.r,self.g,self.b]
    }
    pub fn to_rgba_vec(&self) -> Vec<u8> {
        vec![self.r,self.g,self.b,self.a]
    }
    pub fn to_rgb_vec(&self) -> Vec<u8> {
        vec![self.r,self.g,self.b]
    }
    pub fn to_rgb565_vec(&self) -> Vec<u8> {
        //turn rgba into two adjacent bytes. use set
        let upper = ((self.r >> 3)<<3) | ((self.g & 0b111_00000) >> 5);
        let lower = (((self.g & 0b00011100) >> 2) << 5) | ((self.b & 0b1111_1000) >> 3);
        vec![lower,upper]
    }
    pub fn from_argb_vec(v:&Vec<u8>) -> ARGBColor {
        ARGBColor { a:v[0], r:v[1],g:v[2],b:v[3]}
    }

    pub fn as_layout(&self, layout: &PixelLayout) -> Vec<u8> {
        match layout {
            PixelLayout::RGB565() => {
                self.to_rgb565_vec()
            }
            PixelLayout::ARGB() => {
                self.to_argb_vec()
            }
        }
    }
}


impl ARGBColor {
    pub fn new_rgb(r: u8, g: u8, b: u8) -> ARGBColor {
        ARGBColor { r, g, b, a: 255 }
    }
    pub fn new_argb(a: u8, r: u8, g: u8, b:u8) -> ARGBColor {
        ARGBColor { a,r,g,b}
    }
}

pub const BLACK:ARGBColor = ARGBColor { r: 0, g: 0, b: 0, a: 255 };
pub const WHITE:ARGBColor = ARGBColor { r: 255, g: 255, b: 255, a: 255 };


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloApp {

}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloAppResponse {
    pub app_id:Uuid,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppDisconnected {
    pub app_id:Uuid,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloWindowManager {

}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloWindowManagerResponse {
    pub wm_id:Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrawRectCommand {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub rect:Rect,
    pub color:ARGBColor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrawImageCommand {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub rect:Rect,
    pub buffer:GFXBuffer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenWindowCommand {
    pub window_type:String,
    pub bounds:Rect,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenWindowResponse {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub window_type:String,
    pub bounds:Rect,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum APICommand {
    AppConnect(HelloApp),
    AppConnectResponse(HelloAppResponse),
    AppDisconnected(AppDisconnected),
    Debug(DebugMessage),

    WMConnect(HelloWindowManager),
    WMConnectResponse(HelloWindowManagerResponse),

    OpenWindowCommand(OpenWindowCommand),
    OpenWindowResponse(OpenWindowResponse),

    DrawRectCommand(DrawRectCommand),
    DrawImageCommand(DrawImageCommand),

    KeyDown(KeyDownEvent),
    KeyUp(KeyUpEvent),
    MouseDown(crate::events::MouseDownEvent),
    MouseMove(crate::events::MouseMoveEvent),
    MouseUp(crate::events::MouseUpEvent),

    SystemShutdown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IncomingMessage {
    pub source:Uuid,
    pub command:APICommand,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Point {
    pub x:i32,
    pub y:i32,
}

impl Point {
    pub fn subtract(&self, pt: &Point) -> Point {
        Point {
            x:self.x - pt.x,
            y:self.y - pt.y,
        }
    }
    pub fn add(&self, pt:&Point) -> Point {
        Point::init(self.x + pt.x, self.y + pt.y)
    }
    pub fn copy_from(&mut self, pt:&Point) {
        self.x = pt.x;
        self.y = pt.y;
    }
    pub fn init(x:i32,y:i32) -> Point {
        Point {
            x,
            y
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Size {
    pub w:i32,
    pub h:i32,
}
impl Size {
    pub fn init(w: i32, h: i32) -> Size {
        Size { w, h }
    }
    pub fn grow(&self, p:i32) -> Size {
        Size {
            w: self.w+p,
            h: self.h+p
        }
    }

}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Padding {
    pub left:i32,
    pub right:i32,
    pub top: i32,
    pub bottom: i32,
}

impl Padding {
    pub fn init(top: i32, right: i32, bottom: i32, left: i32) -> Padding {
        Padding {
            left,
            right,
            top,
            bottom
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    pub x:i32,
    pub y:i32,
    pub w:i32,
    pub h:i32,
}

impl Rect {
    pub(crate) fn subtract(&self, pt: &Point) -> Rect {
        Rect {
            x: self.x - pt.x,
            y: self.y - pt.y,
            w: self.w,
            h: self.h
        }
    }
}

impl Rect {
    pub(crate) fn add(&self, pt: &Point) -> Rect {
        Rect {
            x: self.x + pt.x,
            y: self.y + pt.y,
            w: self.w,
            h: self.h
        }
    }
}

impl Rect {
    pub fn grow(&self, pad: &Padding) -> Rect {
        Rect {
            x: self.x - pad.left,
            y: self.y - pad.top,
            w: self.w + pad.left + pad.right,
            h: self.h + pad.top + pad.bottom,
        }
    }
    pub fn from_ints(x:i32, y:i32, w: i32, h:i32) -> Rect {
        Rect {
            x,y,w,h
        }
    }
    pub fn contains(&self, pt: &Point) -> bool {
        if pt.x < self.x { return false }
        if pt.y < self.y { return false }
        if pt.x > self.x + self.w { return false }
        if pt.y > self.y + self.h { return false}
        return true
    }
    pub fn position(&self) -> Point {
        return Point::init(self.x,self.y);
    }
    pub fn size(&self) -> Size {
        return Size::init(self.w,self.h);
    }
    pub fn set_position(&mut self, pos: &Point) {
        self.x = pos.x;
        self.y = pos.y;
    }
    pub fn clamp(&self, pt:&Point) -> Point {
        let mut x = pt.x;
        let mut y = pt.y;
        if x < self.x { x = self.x }
        if y < self.y { y = self.y }
        if x > self.x+self.w { x = self.x+self.w };
        if y > self.y + self.h { y = self.y + self.h; }
        return Point::init(x,y);
    }
    pub(crate) fn intersect(&self, r2: Rect) -> Rect {
        let c1 = self.lower_right_corner();
        let c2 = r2.lower_right_corner();
        let rx = self.x.max(r2.x);
        let ry = self.y.max(r2.y);
        let r2x = c1.x.min(c2.x);
        let r2y = c1.y.min(c2.y);
        Rect {
            x:rx,
            y:ry,
            w:r2x-rx,
            h:r2y-ry,
        }
    }
    fn lower_right_corner(&self) -> Point {
        Point::init(self.x+self.w,self.y+self.h)
    }
    pub(crate) fn is_empty(&self) -> bool {
        if self.w <= 0 {
            return true;
        }
        if self.h <= 0 {
            return true;
        }
        return false;
    }
}
impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{},{}",self.x,self.y).as_str())
    }
}
impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{},{} {}x{}",self.x,self.y, self.w,self.h).as_str())
    }
}


pub const DEBUG_PORT:i32 = 3335;
pub const WINDOW_MANAGER_PORT:i32 = 3334;
pub const APP_MANAGER_PORT:i32 = 3333;



#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DebugMessage {
    HelloDebugger,
    HelloDebuggerResponse,
    ServerStarted,
    ServerStopped,
    WindowManagerConnected,
    WindowManagerDisconnected,
    AppConnected(String),
    AppDisconnected(String),
    WindowOpened(String),
    WindowClosed(String),
    BackgroundReceivedMouseEvent,
    WindowFocusChanged(String),
    RequestServerShutdown,
    AppLog(String),
    FakeMouseEvent(MouseDownEvent),
    ScreenCapture(Rect,String),
    ScreenCaptureResponse(),
}

#[test]
fn test_rect_intersect() {
    let r1 = Rect::from_ints(0,0,500,500);
    let r2 = Rect::from_ints(0,0,250,250);
    let r3 = r1.intersect(r2);
    println!("r3 {}",r3);
    assert_eq!(r3,Rect::from_ints(0,0,250,250));
}
