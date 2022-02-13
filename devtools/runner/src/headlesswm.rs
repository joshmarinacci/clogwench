use common::{APICommand, DebugMessage, HelloWindowManager, IncomingMessage, Point, WINDOW_MANAGER_PORT};
use common_wm::{OutgoingMessage, WindowManagerState};
use core::default::Default;
use core::option::Option;
use core::option::Option::{None, Some};
use core::result::Result::{Err, Ok};
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::thread;
use std::thread::JoinHandle;
use thread::spawn;
use serde::Deserialize;
use common::events::{MouseButton, MouseDownEvent};

pub struct HeadlessWindowManager {
    stream: TcpStream,
}

fn pt(text:&str) {
    println!("HWM: {}",text);
}

impl HeadlessWindowManager {
    pub fn init() -> Option<HeadlessWindowManager> {
        let conn_string = format!("localhost:{}",WINDOW_MANAGER_PORT);

        match TcpStream::connect(conn_string) {
            Ok(stream) => {
                let (tx_out, rx_out) =mpsc::channel::<OutgoingMessage>();
                let (tx_in, rx_in) = mpsc::channel::<IncomingMessage>();
                pt("HWM: connected to the central server");

                let mut state = WindowManagerState::init();
                let mut wm = HeadlessWindowManager { stream };
                let sending_handle = spawn({
                    let mut stream = wm.stream.try_clone().unwrap();
                    move || {
                        loop {
                            for out in &rx_out {
                                pt(&format!("got a message to send back out {:?}", out));
                                let im = IncomingMessage {
                                    source: Default::default(),
                                    command: out.command
                                };
                                pt(&format!("sending out message {:?}", im));
                                let data = serde_json::to_string(&im).unwrap();
                                pt(&format!("sending data {:?}", data));
                                stream.write_all(data.as_ref()).unwrap();
                            }
                        }
                        pt("sending thread is done");
                    }
                });
                let receiving_handle = spawn({
                    let stream = wm.stream.try_clone().unwrap();
                    move || {
                        pt("receiving thread starting");
                        let mut de = serde_json::Deserializer::from_reader(stream);
                        loop {
                            match IncomingMessage::deserialize(&mut de) {
                                Ok(cmd) => {
                                    pt(&format!("received command {:?}", cmd));
                                    match tx_in.send(cmd) {
                                        Ok(_) => {
                                            pt("sent just fine");
                                        }
                                        Err(e) => {
                                            pt("had an error!!");
                                            println!("e {}",e);
                                            // break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    pt(&format!("error deserializing {:?}", e));
                                    break;
                                }
                            }
                        }
                        pt("receiving thread is done");
                    }
                });
                let processing_handle = spawn({
                    let tx_out = tx_out.clone();
                    move || {
                        println!("HWM: reading from rx_in");
                        loop {
                            for cmd in rx_in.try_iter() {
                                pt(&format!("received message {:?}", cmd));
                                match cmd.command {
                                    APICommand::AppConnectResponse(res) => {
                                        state.add_app(res.app_id);
                                    },
                                    APICommand::OpenWindowResponse(ow) => {
                                        state.add_window(ow.app_id, ow.window_id, &ow.bounds);
                                    },
                                    APICommand::DrawRectCommand(_) => {},//ignore draw commands
                                    APICommand::MouseDown(evt) => {
                                        pt("pretending to process a mouse down. lets see what becomes focused?");
                                        let point = Point::init(evt.x, evt.y);
                                        if let Some(win) = state.pick_window_at(point) {
                                            pt("picked a window");
                                            let wid = win.id.clone();
                                            let aid = win.owner.clone();
                                            state.set_focused_window(wid);
                                            tx_out.send(OutgoingMessage {
                                                recipient: Default::default(),
                                                command: APICommand::DebugConnect(DebugMessage::WindowFocusChanged(String::from("foo")))
                                            }).unwrap();
                                            tx_out.send(OutgoingMessage {
                                                recipient: aid,
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
                                            pt("clicked on nothing. sending background debug event");
                                            tx_out.send(OutgoingMessage {
                                                recipient: Default::default(),
                                                command: APICommand::DebugConnect(DebugMessage::BackgroundReceivedMouseEvent)
                                            }).unwrap();
                                        }
                                    }
                                    _ => {
                                        pt(&format!("unhandled message {:?}", cmd));
                                    }
                                };
                            }
                        }
                    }
                });

                let im = OutgoingMessage {
                    recipient: Default::default(),
                    command: APICommand::WMConnect(HelloWindowManager {})
                };
                tx_out.send(im).unwrap();
                pt("window manager fully connected to the central server");
                Some(wm)
            }
            _ => {
                pt(&format!("could not connect to server at"));
                None
            }
        }
    }

    fn log(&self, str: &String) {
        println!("HWM: {}",str);
    }
}


