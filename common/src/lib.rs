use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::events::{KeyDownEvent, KeyUpEvent};

pub mod client;
pub mod events;
pub mod graphics;


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

impl ARGBColor {
    pub(crate) fn as_vec(&self) -> Vec<u8> {
        Vec::from([self.a, self.r, self.g, self.b])
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

    WMConnect(HelloWindowManager),
    WMConnectResponse(HelloWindowManagerResponse),

    OpenWindowCommand(OpenWindowCommand),
    OpenWindowResponse(OpenWindowResponse),

    DrawRectCommand(DrawRectCommand),

    KeyDown(KeyDownEvent),
    KeyUp(KeyUpEvent),
    MouseDown(crate::events::MouseDownEvent),
    MouseMove(crate::events::MouseMoveEvent),
    MouseUp(crate::events::MouseUpEvent),
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
    pub fn add(&self, pt:Point) -> Point {
        Point::init(self.x + pt.x, self.y + pt.y)
    }
    pub fn copy_from(&mut self, pt:Point) {
        self.x = pt.x;
        self.y = pt.y;
    }
}

impl Point {
    pub fn init(x:i32,y:i32) -> Point {
        Point {
            x,
            y
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Rect {
    pub x:i32,
    pub y:i32,
    pub w:i32,
    pub h:i32,
}

impl Rect {
    pub fn contains(&self, pt: Point) -> bool {
        if pt.x < self.x { return false }
        if pt.y < self.y { return false }
        if pt.x > self.x + self.w { return false }
        if pt.y > self.y + self.h { return false}
        return true
    }
}

impl Rect {
    pub fn set_position(&mut self, pos: &Point) {
        self.x = pos.x;
        self.y = pos.y;
    }
}

impl Rect {
    pub fn from_ints(x:i32, y:i32, w: i32, h:i32) -> Rect {
        Rect {
            x,y,w,h
        }
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
}
/*
pub struct Window {
    pub id:Uuid,
    pub bounds:Rect,
}

impl Window {
    pub fn  from_rect(rect: Rect) -> Window {
        Window {
            id: Uuid::new_v4(),
            bounds: rect,
        }
    }
}

pub struct App {
    pub id:Uuid,
    pub connection:TcpStream,
    pub windows:Vec<Window>,
}

impl App {
    pub fn from_stream(stream: TcpStream) -> App {
        App {
            id:Uuid::new_v4(),
            connection: stream.try_clone().unwrap(),
            windows: vec![]
        }
    }
}


pub struct CentralState {
    appmap:HashMap<Uuid,App>
}

impl CentralState {
    pub fn init() -> CentralState {
        CentralState {
            appmap: Default::default()
        }
    }
}

impl CentralState {
    pub fn add_window(&mut self, appid: Uuid, window:Window) {
        if let Some(app) = self.appmap.get_mut(&appid) {
            app.windows.push(window);
        }
    }
    pub fn add_app(&mut self, app:App) {
        self.appmap.insert(app.id,app);
    }
    pub fn app_list(&mut self) -> std::collections::hash_map::IterMut<'_, Uuid, App> {
        self.appmap.iter_mut()
    }
    pub fn find_app_by_id(&mut self, id:Uuid) -> Option<&App> {
        self.appmap.get(&id)
    }
}
*/
