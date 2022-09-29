use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{error, info};
use uuid::Uuid;
use common::{APICommand, CloseWindowResponse, HelloWindowManager, IncomingMessage};
use serde::{Deserialize, Serialize};
use common::events::{MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use gfx::graphics::{ARGBColor, GFXBuffer, PixelLayout, Point, Rect, Size};

pub struct App {
    pub id:Uuid,
    pub windows:Vec<Window>,
}

pub const TITLE_BAR_HEIGHT:i32 = 20;
pub const WINDOW_BORDER_WIDTH:i32 = 2;
pub const WINDOW_COLOR:ARGBColor           = ARGBColor { r: 255, g: 0,   b: 0,   a: 255 };
pub const TITLEBAR_COLOR:ARGBColor         = ARGBColor { r: 0,   g: 255, b: 0,   a: 255 };
pub const WINDOW_BUTTON_COLOR:ARGBColor    = ARGBColor { r: 255, g: 255,   b: 0,   a: 255 };
pub const FOCUSED_WINDOW_COLOR:ARGBColor   = ARGBColor { r: 255, g: 200, b: 200, a: 255 };
pub const FOCUSED_TITLEBAR_COLOR:ARGBColor = ARGBColor { r: 200, g: 255, b: 200, a: 255 };

pub enum WindowType {
    Plain(),
    Popup(),
}
pub struct Window {
    pub id:Uuid,
    pub owner:Uuid,
    pub backbuffer:GFXBuffer,
    pub position:Point,
    pub content_size: Size,
    pub window_type:WindowType,
    pub title:String,
}

impl Window {
    pub(crate) fn set_size(&mut self, size: Size) {
        self.content_size.w = size.w;
        self.content_size.h = size.h;
    }
    pub fn content_bounds(&self) -> Rect {
        Rect {
            x:self.position.x + WINDOW_BORDER_WIDTH,
            y:self.position.y + WINDOW_BORDER_WIDTH + TITLE_BAR_HEIGHT,
            w:self.content_size.w,
            h:self.content_size.h,
        }
    }
    pub fn external_bounds(&self) -> Rect {
        Rect {
            x:self.position.x,
            y:self.position.y,
            w:WINDOW_BORDER_WIDTH+self.content_size.w+WINDOW_BORDER_WIDTH,
            h:WINDOW_BORDER_WIDTH+TITLE_BAR_HEIGHT+self.content_size.h+WINDOW_BORDER_WIDTH,
        }
    }
    pub fn titlebar_bounds(&self) -> Rect {
        Rect {
            x:self.position.x + WINDOW_BORDER_WIDTH,
            y:self.position.y + WINDOW_BORDER_WIDTH,
            w:self.content_size.w,
            h:TITLE_BAR_HEIGHT,
        }
    }
    pub fn resize_bounds(&self) -> Rect {
        Rect {
            x:self.position.x + WINDOW_BORDER_WIDTH+ self.content_size.w - 20,
            y:self.position.y + WINDOW_BORDER_WIDTH+TITLE_BAR_HEIGHT + self.content_size.h - 20,
            w:20,
            h:20,
        }
    }
    pub fn close_button_bounds(&self) -> Rect {
        Rect {
            x:self.position.x + WINDOW_BORDER_WIDTH+1,
            y:self.position.y + WINDOW_BORDER_WIDTH+1,
            w: TITLE_BAR_HEIGHT-2,
            h: TITLE_BAR_HEIGHT-2,
        }
    }
}


pub struct WindowManagerState {
    apps:Vec<App>,
    focused:Option<Uuid>,
    pub preferred_pixel_layout: PixelLayout,
    pub window_order:Vec<Uuid>,
    pub resize_rect:Option<Rect>,
}

impl WindowManagerState {
    pub fn init(ppl:&PixelLayout) -> WindowManagerState {
        WindowManagerState {
            apps: Vec::new(),
            focused: None,
            preferred_pixel_layout:ppl.clone(),
            window_order: vec![],
            resize_rect:None
        }
    }

    pub fn get_windows_in_order(&self) -> Vec<&Window> {
        self.window_order.iter()
            .map(|winid| self.lookup_window(winid.clone()))
            .filter_map(|o|o).collect::<Vec<&Window>>()
    }

    pub fn is_focused_window(&self, win: &Window) -> bool {
        if let Some(foc) = self.get_focused_window() {
            if foc.eq(&win.id) {
                return true
            }
        }
        false
    }
    pub fn add_app(&mut self, app_id: Uuid) {
        let app = App {
            id: app_id,
            windows: vec![]
        };
        self.apps.push(app);
    }
    pub fn find_app(&mut self, app_id: Uuid) -> Option<&mut App> {
        self.apps.iter_mut().find(|a|a.id == app_id)
    }
    pub fn add_window(&mut self, app_id: Uuid, win_id:Uuid, bounds:&Rect, title: &String) -> Uuid {
        let mut win = Window {
            id: win_id,
            position:bounds.position(),
            content_size:bounds.size(),
            owner: app_id,
            backbuffer: GFXBuffer::new(bounds.w as u32, bounds.h as u32, &self.preferred_pixel_layout),
            window_type: WindowType::Plain(),
            title: title.clone(),
        };
        let bg_color:ARGBColor = ARGBColor::new_rgb(255, 128, 0);
        win.backbuffer.clear(&bg_color);
        self.window_order.push(win.id.clone());
        if let Some(app) = self.find_app(app_id) {
            app.windows.push(win);
        }
        win_id
    }

    pub fn get_focused_window(&self) -> &Option<Uuid> {
        &self.focused
    }
    pub fn set_focused_window(&mut self, winid:Uuid) {
        self.focused = *&Some(winid);
    }
    pub fn raise_window(&mut self, win_id: Uuid) {
        if let Some(n) = self.window_order.iter().position(|w|w.eq(&win_id)) {
            self.window_order.remove(n);
            self.window_order.push(win_id);
        }

    }

    pub fn pick_window_at<'a>(&'a self, pt:Point) -> Option<&'a Window> {
        for win_id in self.window_order.iter().rev() {
            for app in &self.apps {
                for win in &app.windows {
                    if win.id.eq(win_id) {
                        if win.external_bounds().contains(&pt) {
                            return Some(win)
                        }
                    }
                }
            }
        }
        None
    }
    pub fn lookup_window_mut<'a>(&'a mut self, win_id: Uuid) -> Option<&'a mut Window> {
        for app in &mut self.apps {
            for win in &mut app.windows {
                if win.id == win_id {
                    return Some(win)
                }
            }
        }
        None
    }
    pub fn lookup_window<'a>(&'a self, win_id: Uuid) -> Option<&'a Window> {
        for app in &self.apps {
            for win in &app.windows {
                if win.id == win_id {
                    return Some(win)
                }
            }
        }
        None
    }
    pub fn dump(&self) {
        info!("WM State");
        for app in &self.apps {
            info!("  app  {}",app.id);
            for win in &app.windows {
                info!("    win {:?} {:?}",win.position, win.content_size)
            }
        }
    }
    pub fn window_list(&self) -> Vec<&Window> {
        let mut res:Vec<&Window> = vec![];
        for app in &self.apps {
            for win in &app.windows {
                res.push(win);
            }
        }
        res
    }
    pub fn window_list_mut(&mut self) -> Vec<&mut Window> {
        let mut res:Vec<&mut Window> = vec![];
        for app in &mut self.apps {
            for win in &mut app.windows {
                res.push(win);
            }
        }
        res
    }
    pub fn remove_window(&mut self, app_id: Uuid, win_id:Uuid) {
        if let Some(app) = self.find_app(app_id) {
            if let Some(n) = app.windows.iter().position(|w| w.id == win_id) {
                println!("removing window from the app windows list");
                app.windows.remove(n);
                if let Some(n) = self.window_order.iter().position(|id|id == &win_id) {
                    self.window_order.remove(n);
                }
            }
        }
    }
    pub fn remove_app(&mut self, app_id: Uuid) {
        if let Some(app) = self.find_app(app_id) {
            app.windows.clear();
        }
        self.apps.retain(|a| a.id != app_id)
    }

}

pub struct CentralConnection {
    pub stream: TcpStream,
    recv_thread: JoinHandle<()>,
    send_thread: JoinHandle<()>,
    pub tx_out: Sender<IncomingMessage>,
}

pub fn start_wm_network_connection(stop: Arc<AtomicBool>, sender: Sender<IncomingMessage>) -> Option<CentralConnection> {
    let conn_string ="localhost:3334";
    match TcpStream::connect(conn_string) {
        Ok(mut master_stream) => {
            //send hello message
            let im = IncomingMessage {
                trace: true,
                timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                source: Default::default(),
                command: APICommand::WMConnect(HelloWindowManager {})};
            match serde_json::to_string(&im) {
                Ok(data) => {
                    // println!("sending data {:?}", data);
                    if let Err(e) = master_stream.write_all(data.as_ref()) {
                        error!("error sending data back to server {}",e);
                        return None
                    }
                }
                Err(e) => {
                    error!("error serializing incoming messages {}",e);
                    return None
                }
            }
            //wait for the response
            let mut de = serde_json::Deserializer::from_reader(&master_stream);
            match IncomingMessage::deserialize(&mut de) {
                Ok(cmd) => {
                    if cmd.trace {
                        info!("==== received command {:?}", cmd);
                    }
                    if let APICommand::WMConnectResponse(res) = cmd.command {
                        // info!("got response back from the server {:?}",res);
                        // res.wm_id
                    }
                }
                Err(e) => {
                    error!("error deserializing {:?}", e);
                    stop.store(true,Ordering::Relaxed);
                    return None
                }
            }
            // info!("window manager fully connected to the central server");

            let (tx_out, rx_out) =mpsc::channel::<IncomingMessage>();
            //receiving thread
            // create thread to read IncomingMessage from network and copy to the WM sender
            let receiving_handle = thread::spawn({
                let stream = master_stream.try_clone().unwrap();
                let stop = stop.clone();
                // let tx_in = tx_in.clone();
                move || {
                    // info!("receiving thread starting");
                    let mut de = serde_json::Deserializer::from_reader(stream);
                    loop {
                        if stop.load(Ordering::Relaxed) == true {
                            break;
                        }
                        match IncomingMessage::deserialize(&mut de) {
                            Ok(cmd) => {
                                // info!("received command {:?}", cmd);
                                if let Err(e) = sender.send(cmd) {
                                    error!("error sending incoming command {}",e);
                                    stop.store(true,Ordering::Relaxed);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("error deserializing {:?}", e);
                                stop.store(true,Ordering::Relaxed);
                                break;
                            }
                        }
                    }
                    info!("receiving thread ending")
                }
            });
            //sending thread
            // create thread to read Messages from rx_out and copy them to the network stream
            let sending_handle = thread::spawn({
                let mut stream = master_stream.try_clone().unwrap();
                let stop = stop.clone();
                move || {
                    // info!("sending thread starting");
                    for out in rx_out {
                        if stop.load(Ordering::Relaxed) == true {
                            break;
                        }
                        // info!("got a message to send back out {:?}",out);
                        let im = IncomingMessage {
                            trace: out.trace,
                            timestamp_usec: out.timestamp_usec,
                            source: Default::default(),
                            command: out.command
                        };
                        if im.trace {
                            info!("sending out message {:?}",im);
                        }
                        match serde_json::to_string(&im) {
                            Ok(data) => {
                                if im.trace {
                                    info!("sending data {:?}", data);
                                }
                                if let Err(e) = stream.write_all(data.as_ref()) {
                                    error!("error sending data back to server {}",e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("error serializing incoming messages {}",e);
                                break;
                            }
                        }
                        // let data = serde_json::to_string(&im)
                        // println!("sending data {:?}", data);
                        // stream.write_all(data.as_ref()).expect("failed to send rect");
                    }
                    info!("sending thread ending");
                }
            });
            Some(CentralConnection {
                stream: master_stream,
                send_thread:sending_handle,
                recv_thread:receiving_handle,
                tx_out,
            })

        }
        _ => {
            error!("could not connect to server at {}",conn_string);
            None
        }
    }
}


pub trait InputGesture {
    fn mouse_down(&mut self, evt:MouseDownEvent, source:&IncomingMessage, state:&mut WindowManagerState, tx_out:&Sender<IncomingMessage>);
    fn mouse_move(&mut self, evt:MouseMoveEvent, state:&mut WindowManagerState, tx_out:&Sender<IncomingMessage>);
    fn mouse_up(  &mut self, evt:MouseUpEvent, state:&mut WindowManagerState, tx_out:&Sender<IncomingMessage>);
}


pub struct NoOpGesture {

}

impl NoOpGesture {
    pub fn init() -> NoOpGesture {
        NoOpGesture {}
    }
}

impl InputGesture for NoOpGesture {
    fn mouse_down(&mut self, evt: MouseDownEvent, source: &IncomingMessage, state: &mut WindowManagerState, tx_out: &Sender<IncomingMessage>) {
        info!("got a mouse down event {:?}",evt);
    }

    fn mouse_move(&mut self, evt: MouseMoveEvent, state:&mut WindowManagerState, tx_out:&Sender<IncomingMessage>) {
        //info!("got a mouse move event {:?}",evt);
    }

    fn mouse_up(&mut self, evt: MouseUpEvent, state:&mut WindowManagerState, tx_out:&Sender<IncomingMessage>) {
        // info!("got a mouse up event {:?}",evt);
    }
}

pub struct WindowDragGesture {
    mouse_start:Point,
    winid:Uuid,
    pub win_start: Point,
}

impl WindowDragGesture {
    pub fn init(start: Point, winid: Uuid) -> WindowDragGesture {
        WindowDragGesture {
            mouse_start:Point::init(0, 0),
            win_start:Point::init(0,0),
            winid
        }
    }
}

impl InputGesture for WindowDragGesture {
    fn mouse_down(&mut self, evt: MouseDownEvent, source: &IncomingMessage, state: &mut WindowManagerState, tx_out: &Sender<IncomingMessage>) {
        // info!("WDG: mouse down {:?}",evt);
        self.win_start = if let Some(win) = state.lookup_window(self.winid) {
            win.position.clone()
        } else {
            panic!("we can't find the window! {}",self.winid);
        };
        self.mouse_start = Point::init(evt.x, evt.y);
    }

    fn mouse_move(&mut self, evt: MouseMoveEvent, state:&mut WindowManagerState, tx_out:&Sender<IncomingMessage>) {
        // info!("WDG: mouse move {:?}",evt);
        let curr = Point::init(evt.x,evt.y);
        let diff = self.mouse_start.subtract(&self.win_start);
        let new_pos = curr.subtract(&diff);
        // info!("dragging window {} by {:?}",self.winid,diff);
        if let Some(win) = state.lookup_window_mut(self.winid) {
            win.position.copy_from(&new_pos);
        }
    }

    fn mouse_up(&mut self, evt: MouseUpEvent, state:&mut WindowManagerState, tx_out:&Sender<IncomingMessage>) {
        // info!("WDG completed");
        let curr = Point::init(evt.x,evt.y);
        let diff = self.mouse_start.subtract(&self.win_start);
        let new_pos = curr.subtract(&diff);
        // info!("new window position is {} to {:?}",self.winid,curr);
        if let Some(win) = state.lookup_window_mut(self.winid) {
            win.position.copy_from(&new_pos);
        }
    }
}

pub struct WindowResizeGesture {
    mouse_start:Point,
    winid:Uuid,
    win_start: Point,
}

impl WindowResizeGesture {
    pub fn init(start:Point, winid:Uuid) -> WindowResizeGesture {
        WindowResizeGesture {
            mouse_start:Point::init(0, 0),
            win_start:Point::init(0,0),
            winid
        }
    }
}
impl InputGesture for WindowResizeGesture {
    fn mouse_down(&mut self, evt: MouseDownEvent, source: &IncomingMessage, state: &mut WindowManagerState, tx_out: &Sender<IncomingMessage>) {
        // println!("mouse down on resize {},{}", evt.x, evt.y);
        self.mouse_start.x = evt.x;
        self.mouse_start.y = evt.y;
    }

    fn mouse_move(&mut self, evt: MouseMoveEvent, state: &mut WindowManagerState, tx_out: &Sender<IncomingMessage>) {
        // println!("mouse move on resize {},{}", evt.x, evt.y);
        if let Some(win) = state.lookup_window(self.winid) {
            let wb = win.external_bounds();
            state.resize_rect = Some(Rect::from_ints(wb.x,wb.y,evt.x-wb.x,evt.y-wb.y));
        }
    }

    fn mouse_up(&mut self, evt: MouseUpEvent, state: &mut WindowManagerState, tx_out: &Sender<IncomingMessage>) {
        // println!("mouse up on resize {},{}",evt.x,evt.y);
        state.resize_rect = None;
        if let Some(win) = state.lookup_window_mut(self.winid) {
            let diff = Point::init(evt.x,evt.y).subtract(&win.position);
            let tb_h = win.titlebar_bounds().h;
            // win.position.copy_from(&new_pos);
            win.set_size(Size{ w: diff.x, h: diff.y -tb_h })
        }
    }
}

pub struct WindowCloseButtonGesture {
    pub winid: Uuid,
}
impl WindowCloseButtonGesture {
    pub fn init(start:Point, winid:Uuid) -> WindowCloseButtonGesture {
        WindowCloseButtonGesture {
            // mouse_start:Point::init(0, 0),
            // win_start:Point::init(0,0),
            winid
        }
    }
}
impl InputGesture for WindowCloseButtonGesture {
    fn mouse_down(&mut self, evt: MouseDownEvent, source: &IncomingMessage, state: &mut WindowManagerState, tx_out: &Sender<IncomingMessage>) {
    }

    fn mouse_move(&mut self, evt: MouseMoveEvent, state: &mut WindowManagerState, tx_out: &Sender<IncomingMessage>) {
    }

    fn mouse_up(&mut self, evt: MouseUpEvent, state: &mut WindowManagerState, tx_out: &Sender<IncomingMessage>) {
        // println!("mouse up. send the window close event");
        // let point = Point::init(evt.x, evt.y);
        if let Some(win) = state.lookup_window(self.winid) {
            // info!("picked a window for mouse up");
            let wid = win.id.clone();
            let aid = win.owner.clone();
            // let app_point = point.subtract(&win.content_bounds().position());
            state.remove_window(aid,wid);
            tx_out.send(IncomingMessage {
                source:Default::default(),
                trace: false,
                timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                command: APICommand::CloseWindowResponse(CloseWindowResponse {
                    app_id: aid,
                    window_id: wid,
                })
            }).unwrap();
        }
    }
}
pub struct AppMouseGesture {
    pub winid: Uuid,
    pub app_id: Uuid,
}

impl AppMouseGesture {
    pub fn init(app_id:Uuid, winid: Uuid) -> AppMouseGesture {
        AppMouseGesture {
            winid,
            app_id,
        }
    }
}


impl InputGesture for AppMouseGesture {
    fn mouse_down(&mut self, evt: MouseDownEvent, source: &IncomingMessage, state: &mut WindowManagerState, tx_out: &Sender<IncomingMessage>) {
        // println!("Mouse down to app. trace is {}",source.trace);
        let point = Point::init(evt.x, evt.y);
        if let Some(win) = state.lookup_window(self.winid) {
            let app_point = point.subtract(&win.content_bounds().position());
            // win.position.copy_from(&new_pos);
            tx_out.send(IncomingMessage {
                source:Default::default(),
                trace:source.trace,
                timestamp_usec:source.timestamp_usec,
                command: APICommand::MouseDown(MouseDownEvent {
                    app_id: self.app_id,
                    window_id: self.winid,
                    original_timestamp: evt.original_timestamp,
                    button: MouseButton::Primary,
                    x: app_point.x,
                    y: app_point.y
                })
            }).unwrap();
        }

    }

    fn mouse_move(&mut self, evt: MouseMoveEvent, state: &mut WindowManagerState, tx_out:&Sender<IncomingMessage>) {
        println!("Mouse move to app");
        let point = Point::init(evt.x, evt.y);
        if let Some(win) = state.pick_window_at(point) {
            // info!("picked a window for mouse move");
            let wid = win.id.clone();
            let aid = win.owner.clone();
            let app_point = point.subtract(&win.content_bounds().position());
            tx_out.send(IncomingMessage {
                source:Default::default(),
                trace: false,
                timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                command: APICommand::MouseMove(MouseMoveEvent {
                    app_id: aid,
                    window_id: wid,
                    original_timestamp: evt.original_timestamp,
                    button: MouseButton::Primary,
                    x: app_point.x,
                    y: app_point.y
                })
            }).unwrap();
        }
    }

    fn mouse_up(&mut self, evt: MouseUpEvent, state: &mut WindowManagerState, tx_out:&Sender<IncomingMessage>) {
        println!("Mouse up to app");
        let point = Point::init(evt.x, evt.y);
        if let Some(win) = state.pick_window_at(point) {
            // info!("picked a window for mouse up");
            let wid = win.id.clone();
            let aid = win.owner.clone();
            let app_point = point.subtract(&win.content_bounds().position());
            tx_out.send(IncomingMessage {
                source:Default::default(),
                trace: false,
                timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                command: APICommand::MouseUp(MouseUpEvent {
                    app_id: aid,
                    window_id: wid,
                    original_timestamp: evt.original_timestamp,
                    button: MouseButton::Primary,
                    x: app_point.x,
                    y: app_point.y
                })
            }).unwrap();
        }
    }
}
