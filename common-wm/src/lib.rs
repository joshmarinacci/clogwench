use std::error::Error;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use log::{error, info};
use uuid::Uuid;
use common::{APICommand, ARGBColor, BLACK, HelloWindowManager, IncomingMessage, Padding, Point, Rect, Size};
use serde::{Deserialize, Serialize};
use common::events::{MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use common::graphics::ColorDepth::{CD24, CD32};
use common::graphics::GFXBuffer;
use common::graphics::PixelLayout::ARGB;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}


pub struct App {
    pub id:Uuid,
    pub windows:Vec<Window>,
}

pub const TITLE_BAR_HEIGHT:i32 = 10;
pub const WINDOW_BORDER_WIDTH:i32 = 5;
pub const WINDOW_COLOR:ARGBColor           = ARGBColor { r: 255, g: 0,   b: 0,   a: 255 };
pub const TITLEBAR_COLOR:ARGBColor         = ARGBColor { r: 0,   g: 255, b: 0,   a: 255 };
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
    pub window_type:WindowType
}

impl Window {
    pub fn content_bounds(&self) -> Rect {
        return Rect {
            x:self.position.x + WINDOW_BORDER_WIDTH,
            y:self.position.y + WINDOW_BORDER_WIDTH + TITLE_BAR_HEIGHT,
            w:self.content_size.w,
            h:self.content_size.h,
        }
    }
    pub fn external_bounds(&self) -> Rect {
        return Rect {
            x:self.position.x,
            y:self.position.y,
            w:WINDOW_BORDER_WIDTH+self.content_size.w+WINDOW_BORDER_WIDTH,
            h:WINDOW_BORDER_WIDTH+TITLE_BAR_HEIGHT+self.content_size.h+WINDOW_BORDER_WIDTH,
        }
    }
    pub fn titlebar_bounds(&self) -> Rect {
        return Rect {
            x:self.position.x + WINDOW_BORDER_WIDTH,
            y:self.position.y + WINDOW_BORDER_WIDTH,
            w:self.content_size.w,
            h:TITLE_BAR_HEIGHT,
        }
    }
}


pub struct WindowManagerState {
    apps:Vec<App>,
    focused:Option<Uuid>,
}

impl WindowManagerState {
    pub fn init() -> WindowManagerState {
        WindowManagerState {
            apps: Vec::new(),
            focused: None,
        }
    }

    pub fn is_focused_window(&self, win: &Window) -> bool {
        if let Some(foc) = self.get_focused_window() {
            if foc.eq(&win.id) {
                return true
            }
        }
        return false
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
    pub fn add_window(&mut self, app_id: Uuid, win_id:Uuid, bounds:&Rect) -> Uuid {
        let mut win = Window {
            id: win_id,
            position:bounds.position(),
            content_size:bounds.size(),
            owner: app_id,
            backbuffer: GFXBuffer::new(CD32(), bounds.w as u32, bounds.h as u32, ARGB()),
            window_type: WindowType::Plain()
        };
        let BG_COLOR:ARGBColor = ARGBColor::new_rgb(255,128,0);
        win.backbuffer.clear(&BG_COLOR);
        if let Some(app) = self.find_app(app_id) {
            app.windows.push(win);
        }
        return win_id;
    }
    pub fn find_first_window(&self) -> Option<&Window> {
        if !self.apps.is_empty() {
            let app = &self.apps[0];
            if !app.windows.is_empty() {
                return Some(&app.windows[0])
            }
        }
        return None
    }
    pub fn get_focused_window(&self) -> &Option<Uuid> {
        &self.focused
    }
    pub fn set_focused_window(&mut self, winid:Uuid) {
        self.focused = *&Some(winid);
    }

    pub fn pick_window_at<'a>(&'a self, pt: Point) -> Option<&'a Window> {
        for app in &self.apps {
            for win in &app.windows {
                if win.external_bounds().contains(pt) {
                    return Some(win)
                }
            }
        }
        return None
    }
    pub fn lookup_window<'a>(&'a mut self, win_id: Uuid) -> Option<&'a mut Window> {
        for app in &mut self.apps {
            for win in &mut app.windows {
                if win.id == win_id {
                    return Some(win)
                }
            }
        }
        return None
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
        return res;
    }
    pub fn remove_app(&mut self, app_id: Uuid) {
        if let Some(app) = self.find_app(app_id) {
            app.windows.clear();
        }
        self.apps.retain(|a| a.id != app_id)
    }

}
/*
- common abstraction for window managers:
	- windows (with simple memory back buffers). Needs to track window size and position too.
	- window contents, so titlebar is outside of the requested size.
	- an in memory drawing surface.
	- ordering of windows
	- given a point pick the highest window which contains that point
	- get and sent currently focused window, if any.
		- real wm can send keyboard event to the currently focused window.

 */


#[derive(Serialize, Deserialize, Debug)]
pub struct OutgoingMessage {
    pub recipient:Uuid,
    pub command:APICommand,
}

pub struct CentralConnection {
    pub stream: TcpStream,
    recv_thread: JoinHandle<()>,
    send_thread: JoinHandle<()>,
    pub tx_out: Sender<OutgoingMessage>,
    // pub rx_in: Receiver<IncomingMessage>,
    // pub tx_in: Sender<IncomingMessage>,
}

// fn send_hello(sender: Sender<IncomingMessage>, tx_out: Sender<OutgoingMessage>) -> Result<(),String> {
//     //send hello window manager
//     let msg = OutgoingMessage {
//         recipient: Default::default(),
//         command: APICommand::WMConnect(HelloWindowManager {})
//     };
//     tx_out.send(msg).map_err(|e|e.to_string())?;
//
//     let resp = rx_in.recv().map_err(|e|e.to_string())?;
//     let selfid = if let APICommand::WMConnectResponse(res) = resp.command {
//         info!("got response back from the server {:?}",res);
//         res.wm_id
//     } else {
//         panic!("did not get the window manager connect response. gah!");
//     };
//     Ok(())
// }

pub fn start_wm_network_connection(stop: Arc<AtomicBool>, sender: Sender<IncomingMessage>) -> Option<CentralConnection> {
    let conn_string ="localhost:3334";
    match TcpStream::connect(conn_string) {
        Ok(mut master_stream) => {
            let (tx_out, rx_out) =mpsc::channel::<OutgoingMessage>();
            println!("connected to the linux-wm");

            //do the hello connection
            // let msg = OutgoingMessage {
            //     recipient: Default::default(),
            //     command: APICommand::WMConnect(HelloWindowManager {})
            // };
            let im = IncomingMessage { source: Default::default(), command: APICommand::WMConnect(HelloWindowManager {})};
            println!("sending out message {:?}",im);
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
                    info!("received command {:?}", cmd);
                    if let APICommand::WMConnectResponse(res) = cmd.command {
                        info!("got response back from the server {:?}",res);
                        // res.wm_id
                    }
                }
                Err(e) => {
                    error!("error deserializing {:?}", e);
                    stop.store(true,Ordering::Relaxed);
                    return None
                }
            }
            info!("window manager fully connected to the central server");

            //receiving thread
            let receiving_handle = thread::spawn({
                let stream = master_stream.try_clone().unwrap();
                let stop = stop.clone();
                // let tx_in = tx_in.clone();
                move || {
                    info!("receiving thread starting");
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
            let sending_handle = thread::spawn({
                let mut stream = master_stream.try_clone().unwrap();
                let stop = stop.clone();
                move || {
                    info!("sending thread starting");
                    for out in rx_out {
                        if stop.load(Ordering::Relaxed) == true {
                            break;
                        }
                        info!("got a message to send back out {:?}",out);
                        let im = IncomingMessage {
                            source: Default::default(),
                            command: out.command
                        };
                        println!("sending out message {:?}",im);
                        match serde_json::to_string(&im) {
                            Ok(data) => {
                                println!("sending data {:?}", data);
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
                // tx_in:tx_in,
                // rx_in:rx_in,
                tx_out:tx_out,
            })

        }
        _ => {
            error!("could not connect to server at {}",conn_string);
            None
        }
    }
}


pub trait InputGesture {
    fn mouse_down(&mut self, evt:MouseDownEvent, state:&mut WindowManagerState);
    fn mouse_move(&mut self, evt:MouseMoveEvent, state:&mut WindowManagerState);
    fn mouse_up(  &mut self, evt:MouseUpEvent, state:&mut WindowManagerState);
}


pub struct NoOpGesture {

}

impl NoOpGesture {
    pub fn init() -> NoOpGesture {
        NoOpGesture {}
    }
}

impl InputGesture for NoOpGesture {
    fn mouse_down(&mut self, evt: MouseDownEvent, state:&mut WindowManagerState) {
        info!("got a mouse down event {:?}",evt);
    }

    fn mouse_move(&mut self, evt: MouseMoveEvent, state:&mut WindowManagerState) {
        //info!("got a mouse move event {:?}",evt);
    }

    fn mouse_up(&mut self, evt: MouseUpEvent, state:&mut WindowManagerState) {
        info!("got a mouse up event {:?}",evt);
    }
}

pub struct WindowDragGesture {
    start:Point,
    winid:Uuid,
}

impl WindowDragGesture {
    pub fn init(start: Point, win: Uuid) -> WindowDragGesture {
        WindowDragGesture {
            start:Point::init(0,0),
            winid:win
        }
    }
}

impl InputGesture for WindowDragGesture {
    fn mouse_down(&mut self, evt: MouseDownEvent, state:&mut WindowManagerState) {
        info!("WDG: mouse down {:?}",evt);
        self.start = Point::init(evt.x,evt.y);
    }

    fn mouse_move(&mut self, evt: MouseMoveEvent, state:&mut WindowManagerState) {
        info!("WDG: mouse move {:?}",evt);
        let curr = Point::init(evt.x,evt.y);
        let diff = curr.subtract(self.start);
        info!("dragging window {} by {:?}",self.winid,diff);
        if let Some(win) = state.lookup_window(self.winid) {
            win.position.x = curr.x;
            win.position.y = curr.y;
        }
    }

    fn mouse_up(&mut self, evt: MouseUpEvent, state:&mut WindowManagerState) {
        info!("WDG completed");
        let curr = Point::init(evt.x,evt.y);
        info!("new window position is {} to {:?}",self.winid,curr);
        if let Some(win) = state.lookup_window(self.winid) {
            win.position.x = curr.x;
            win.position.y = curr.y;
        }
    }
}

