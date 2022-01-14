use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use log::{error, info};
use uuid::Uuid;
use common::{APICommand, ARGBColor, BLACK, IncomingMessage, Point, Rect, Size};
use serde::{Deserialize, Serialize};
use common::graphics::ColorDepth::CD24;
use common::graphics::GFXBuffer;

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
    windows:Vec<Window>,
}

const TITLE_BAR_HEIGHT:i32 = 10;
const WINDOW_BORDER_WIDTH:i32 = 5;
const WINDOW_COLOR:ARGBColor = ARGBColor { r: 100, g: 0, b: 0, a: 255 };
const TITLEBAR_COLOR:ARGBColor = ARGBColor { r: 250, g: 100, b: 50, a: 255 };
const FOCUSED_WINDOW_COLOR:ARGBColor = ARGBColor { r: 255, g: 255, b: 255, a: 255 };
const FOCUSED_TITLEBAR_COLOR:ARGBColor = ARGBColor { r: 255, g: 200, b: 200, a: 255 };

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
            x:self.position.x,
            y:self.position.y,
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
            focused: None
        }
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
    pub fn add_window(&mut self, app_id: Uuid, win_id:Uuid, bounds:&Rect) {
        let win = Window {
            id: win_id,
            position:bounds.position(),
            content_size:bounds.size(),
            owner: app_id,
            backbuffer: GFXBuffer::new(CD24(), bounds.w as u32, bounds.h as u32),
            window_type: WindowType::Plain()
        };
        if let Some(app) = self.find_app(app_id) {
            app.windows.push(win);
        }
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
        println!("WM State");
        for app in &self.apps {
            println!("  app  {}",app.id);
            for win in &app.windows {
                println!("    win {:?}",win.bounds)
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
    pub rx_in: Receiver<IncomingMessage>,
    pub tx_in: Sender<IncomingMessage>,
}

pub fn start_wm_network_connection(stop: Arc<AtomicBool>) -> Option<CentralConnection> {
    let conn_string ="localhost:3334";
    match TcpStream::connect(conn_string) {
        Ok(master_stream) => {
            let (tx_in, rx_in) = mpsc::channel::<IncomingMessage>();
            let (tx_out, rx_out) =mpsc::channel::<OutgoingMessage>();
            println!("connected to the linux-wm");
            //receiving thread
            let receiving_handle = thread::spawn({
                let stream = master_stream.try_clone().unwrap();
                let stop = stop.clone();
                let tx_in = tx_in.clone();
                move || {
                    info!("receiving thread starting");
                    let mut de = serde_json::Deserializer::from_reader(stream);
                    loop {
                        if stop.load(Ordering::Relaxed) { break; }
                        match IncomingMessage::deserialize(&mut de) {
                            Ok(cmd) => {
                                // info!("received command {:?}", cmd);
                                tx_in.send(cmd).unwrap();
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
                        if stop.load(Ordering::Relaxed) { break; }
                        let im = IncomingMessage {
                            source: Default::default(),
                            command: out.command
                        };
                        println!("sending out message {:?}",im);
                        let data = serde_json::to_string(&im).unwrap();
                        println!("sending data {:?}", data);
                        stream.write_all(data.as_ref()).expect("failed to send rect");
                    }
                    info!("sending thread ending");
                }
            });
            Some(CentralConnection {
                stream: master_stream,
                send_thread:sending_handle,
                recv_thread:receiving_handle,
                tx_in:tx_in,
                rx_in:rx_in,
                tx_out:tx_out,
            })

        }
        _ => {
            error!("could not connect to server at {}",conn_string);
            None
        }
    }
}

