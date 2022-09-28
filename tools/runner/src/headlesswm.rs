use common::{APICommand, DebugMessage, HelloWindowManager, IncomingMessage, Point, WHITE, WINDOW_MANAGER_PORT};
use common_wm::{WindowManagerState};
use core::default::Default;
use core::option::Option;
use core::option::Option::{None, Some};
use core::result::Result::{Err, Ok};
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::{SendError};
use std::thread;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};
use thread::spawn;
use log::{error, info, warn};
use serde::Deserialize;
use common::events::{MouseButton, MouseDownEvent};
use common::graphics::{GFXBuffer, PixelLayout};

pub struct HeadlessWindowManager {
    stream: TcpStream,
    pub(crate) handle:JoinHandle<()>,
}

impl HeadlessWindowManager {
    pub fn init(w: u32, h: u32) -> Option<HeadlessWindowManager> {
        let mut buf = GFXBuffer::new( w, h, &PixelLayout::ARGB());
        buf.clear(&WHITE);
        let conn_string = format!("localhost:{}",WINDOW_MANAGER_PORT);

        match TcpStream::connect(conn_string) {
            Ok(stream) => {
                let (tx_out, rx_out) =mpsc::channel::<IncomingMessage>();
                let (tx_in, rx_in) = mpsc::channel::<IncomingMessage>();
                info!("connected to the central server");

                let mut state = WindowManagerState::init(&PixelLayout::ARGB());
                let sending_handle = spawn({
                    let mut stream = stream.try_clone().unwrap();
                    move || {
                        loop {
                            for out in &rx_out {
                                // pt(&format!("got a message to send back out {:?}", out));
                                let im = IncomingMessage {
                                    trace: false,
                                    timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                                    source: Default::default(),
                                    command: out.command
                                };
                                info!("sending {:?}", im);
                                let data = serde_json::to_string(&im).unwrap();
                                // pt(&format!("sending data {:?}", data));
                                stream.write_all(data.as_ref()).unwrap();
                            }
                        }
                        info!("sending thread is done");
                    }
                });
                let receiving_handle = spawn({
                    let stream = stream.try_clone().unwrap();
                    move || {
                        info!("receiving thread starting");
                        let mut de = serde_json::Deserializer::from_reader(stream);
                        loop {
                            match IncomingMessage::deserialize(&mut de) {
                                Ok(cmd) => {
                                    // pt(&format!("received command {:?}", cmd));
                                    match tx_in.send(cmd) {
                                        Ok(_) => {
                                            // pt("sent just fine");
                                        }
                                        Err(e) => {
                                            error!("had an error!! {}",e);
                                            // break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("error deserializing {:?}", e);
                                    break;
                                }
                            }
                        }
                        info!("receiving thread is done");
                    }
                });
                let handle = spawn({
                    let tx_out = tx_out.clone();
                    move || {
                        info!("reading from rx_in");
                        loop {
                            for cmd in rx_in.try_iter() {
                                // info!("received message {:?}", cmd);
                                match cmd.command {
                                    APICommand::WMConnectResponse(res) => {
                                        // info!("got response for connecting");
                                    },
                                    APICommand::AppConnectResponse(res) => {
                                        state.add_app(res.app_id);
                                    },
                                    APICommand::OpenWindowResponse(ow) => {
                                        state.add_window(ow.app_id, ow.window_id, &ow.bounds,&ow.window_title );
                                    },
                                    APICommand::DrawRectCommand(dr) => {
                                        if let Some(mut win) = state.lookup_window_mut(dr.window_id) {
                                            // info!("draw rect to window {:?} {:?}",&dr.rect, &dr.color);
                                            win.backbuffer.fill_rect(dr.rect, &dr.color);
                                            buf.draw_image(&win.position, &win.backbuffer.bounds(), &win.backbuffer);
                                        }
                                    },
                                    APICommand::DrawImageCommand(di) => {
                                        if let Some(mut win) = state.lookup_window_mut(di.window_id) {
                                            win.backbuffer.fill_rect_with_image(&di.rect, &di.buffer);
                                            buf.draw_image(&win.position, &win.backbuffer.bounds(), &win.backbuffer);
                                        }
                                    }
                                    APICommand::MouseDown(evt) => {
                                        // info!("pretending to process a mouse down. lets see what becomes focused?");
                                        let point = Point::init(evt.x, evt.y);
                                        if let Some(win) = state.pick_window_at(point) {
                                            // info!("picked a window");
                                            let wid = win.id.clone();
                                            let aid = win.owner.clone();
                                            state.set_focused_window(wid);
                                            tx_out.send(IncomingMessage {
                                                source:Default::default(),
                                                trace: false,
                                                timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                                                // recipient: Default::default(),
                                                command: APICommand::Debug(DebugMessage::WindowFocusChanged(String::from("foo")))
                                            }).unwrap();
                                            tx_out.send(IncomingMessage {
                                                source:Default::default(),
                                                trace: false,
                                                timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                                                // recipient: aid,
                                                command: APICommand::MouseDown(MouseDownEvent{
                                                    app_id: aid,
                                                    window_id: wid,
                                                    original_timestamp: evt.original_timestamp,
                                                    button: MouseButton::Primary,
                                                    x: evt.x,
                                                    y: evt.y
                                                })
                                            }).unwrap();
                                        } else {
                                            // info!("clicked on nothing. sending background debug event");
                                            tx_out.send(IncomingMessage {
                                                source:Default::default(),
                                                trace: false,
                                                timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                                                // recipient: Default::default(),
                                                command: APICommand::Debug(DebugMessage::BackgroundReceivedMouseEvent)
                                            }).unwrap();
                                        }
                                    }
                                    APICommand::Debug(DebugMessage::ScreenCapture(rect, str)) => {
                                        let pth = PathBuf::from("./screencapture.png");
                                        // info!("rect for screen capture {:?}",pth);
                                        buf.to_png(&pth);
                                        tx_out.send(IncomingMessage {
                                            source:Default::default(),
                                            trace: false,
                                            timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                                            // recipient: Default::default(),
                                            command: APICommand::Debug(DebugMessage::ScreenCaptureResponse()),
                                        }).unwrap();
                                    }
                                    APICommand::SystemShutdown => {
                                        info!("the core is shutting down. bye");
                                        return;
                                    }
                                    _ => {
                                        warn!("unhandled message {:?}", cmd);
                                    }
                                };
                            }
                        }
                        info!("processing thread ending");
                    }
                });

                let im = IncomingMessage {
                    source:Default::default(),
                    trace: false,
                    timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                    // recipient: Default::default(),
                    command: APICommand::WMConnect(HelloWindowManager {})
                };
                tx_out.send(im).unwrap();
                info!("window manager fully connected to the central server");
                Some(HeadlessWindowManager {
                    stream,
                    handle,
                })
            }
            _ => {
                info!("could not connect to server at");
                None
            }
        }
    }

    fn log(&self, str: &String) {
        println!("HWM: {}",str);
    }
}


