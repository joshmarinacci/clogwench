use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{JoinHandle, spawn};
use serde::Deserialize;
use common::{APICommand, DebugMessage, HelloWindowManager, IncomingMessage, Point, Rect, WINDOW_MANAGER_PORT};
use common::events::{MouseButton, MouseDownEvent};
use common::graphics::export_to_png;
use common_wm::{OutgoingMessage, WindowManagerState};
use plat::{make_plat, Plat};

pub struct PlatformWindowManager {
    pub stream: TcpStream,
    pub plat: Plat,
    pub state: WindowManagerState,
    pub receiving_handle: JoinHandle<()>,
    pub sending_handle: JoinHandle<()>,
    pub tx_out: Sender<OutgoingMessage>,
    pub rx_in: Receiver<IncomingMessage>,
}

fn pt(text:&str) {
    println!("Native WM: {}",text);
}

impl PlatformWindowManager {
    pub(crate) fn init(w: i32, h: i32) -> Option<PlatformWindowManager> {
        let conn_string = format!("localhost:{}",WINDOW_MANAGER_PORT);


        match TcpStream::connect(conn_string) {
            Ok(stream) => {
                pt("HWM: connected to the central server");

                let (tx_out, rx_out) =mpsc::channel::<OutgoingMessage>();
                let (tx_in, mut rx_in) = mpsc::channel::<IncomingMessage>();
                let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

                let sending_handle = spawn({
                    let mut stream = stream.try_clone().unwrap();
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
                    let stream = stream.try_clone().unwrap();
                    let tx_in = tx_in.clone();
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

                // let mut tx_out = tx_out.clone();
                // let tx_in = tx_in.clone();
                // let mut plat = make_plat(stop.clone(), tx_in).unwrap();

                Some(PlatformWindowManager{
                    stream,
                    state:WindowManagerState::init(),
                    plat: make_plat(stop.clone(), tx_in.clone()).unwrap(),
                    sending_handle,
                    receiving_handle,
                    tx_out,
                    rx_in,
                })
            }
            _ => {
                pt(&format!("could not connect to server at"));
                None
            }
        }

    }
}

pub fn main_service_loop(state: &mut WindowManagerState, plat: &mut Plat, rx_in: &mut Receiver<IncomingMessage>, tx_out: &mut Sender<OutgoingMessage>) -> bool {
    plat.service_input();
    for cmd in rx_in.try_iter() {
        pt(&format!("received message {:?}", cmd));
        match cmd.command {
            APICommand::AppConnectResponse(res) => {
                state.add_app(res.app_id);
            },
            APICommand::OpenWindowResponse(ow) => {
                state.add_window(ow.app_id, ow.window_id, &ow.bounds);
            },
            APICommand::DrawRectCommand(dr) => {
                if let Some(mut win) = state.lookup_window(dr.window_id) {
                    println!("draw rect to window {:?} {:?}",&dr.rect, &dr.color);
                    win.backbuffer.fill_rect(dr.rect, &dr.color);
                    // buf.copy_from(win.position.x, win.position.y, &win.backbuffer);
                }
            },
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
                        command: APICommand::Debug(DebugMessage::WindowFocusChanged(String::from("foo")))
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
                        command: APICommand::Debug(DebugMessage::BackgroundReceivedMouseEvent)
                    }).unwrap();
                }
            }
            APICommand::Debug(DebugMessage::ScreenCapture(rect, str)) => {
                let pth = PathBuf::from("./screencapture.png");
                println!("rect for screen capture {:?}",pth);
                // export_to_png(&buf, &pth);
                tx_out.send(OutgoingMessage {
                    recipient: Default::default(),
                    command: APICommand::Debug(DebugMessage::ScreenCaptureResponse()),
                }).unwrap();
            }
            _ => {
                pt(&format!("unhandled message {:?}", cmd));
            }
        };
    }
    plat.service_loop();
    false
}
