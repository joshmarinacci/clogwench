use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::{JoinHandle, spawn};
use std::time::Duration;
use log::info;
use serde::Deserialize;
use common::{APICommand, ARGBColor, DebugMessage, HelloWindowManager, IncomingMessage, Point, Rect, WINDOW_MANAGER_PORT};
use common::events::{KeyCode, KeyDownEvent, MouseButton, MouseDownEvent};
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

impl PlatformWindowManager {
    pub(crate) fn init(w: i32, h: i32) -> Option<PlatformWindowManager> {
        let conn_string = format!("localhost:{}",WINDOW_MANAGER_PORT);


        match TcpStream::connect(conn_string) {
            Ok(stream) => {
                info!("connected to the central server");

                let (tx_out, rx_out) =mpsc::channel::<OutgoingMessage>();
                let (tx_in, mut rx_in) = mpsc::channel::<IncomingMessage>();
                let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

                let sending_handle = spawn({
                    let mut stream = stream.try_clone().unwrap();
                    move || {
                        loop {
                            for out in &rx_out {
                                // info!("got a message to send back out {:?}", out);
                                let im = IncomingMessage {
                                    source: Default::default(),
                                    command: out.command
                                };
                                // info!("sending out message {:?}", im);
                                let data = serde_json::to_string(&im).unwrap();
                                // info!("sending data {:?}", data);
                                stream.write_all(data.as_ref()).unwrap();
                            }
                        }
                        info!("sending thread is done");
                    }
                });
                let receiving_handle = spawn({
                    let stream = stream.try_clone().unwrap();
                    let tx_in = tx_in.clone();
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
                                            info!("had an error!!");
                                            println!("err {}",e);
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    info!("error deserializing {:?}", e);
                                    break;
                                }
                            }
                        }
                        info!("receiving thread is done");
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
                info!("could not connect to server at");
                None
            }
        }

    }
}

pub fn main_service_loop(state: &mut WindowManagerState, plat: &mut Plat, rx_in: &mut Receiver<IncomingMessage>, tx_out: &mut Sender<OutgoingMessage>) -> bool {
    // println!("Native WM service loop");
    plat.service_input();
    for cmd in rx_in.try_iter() {
        // pt(&format!("received {:?}", cmd));
        match cmd.command {
            APICommand::SystemShutdown => {
                info!("the core is shutting down. bye");
                return false;
            }
            APICommand::AppConnectResponse(res) => {
                state.add_app(res.app_id);
            },
            APICommand::OpenWindowResponse(ow) => {
                let win_id = state.add_window(ow.app_id, ow.window_id, &ow.bounds);
                if let Some(win) = state.lookup_window(win_id) {
                    plat.register_image2(&win.backbuffer);
                }
            },
            APICommand::DrawRectCommand(dr) => {
                if let Some(mut win) = state.lookup_window(dr.window_id) {
                    println!("NativeWM: draw rect to window {:?} {:?}",&dr.rect, &dr.color);
                    win.backbuffer.fill_rect(dr.rect, &dr.color);
                    // buf.copy_from(win.position.x, win.position.y, &win.backbuffer);
                }
            },
            APICommand::MouseUp(evt) => {

            },
            APICommand::MouseMove(evt) => {
                //ignore mouse move
            }
            APICommand::MouseDown(evt) => {
                let point = Point::init(evt.x, evt.y);
                if let Some(win) = state.pick_window_at(point) {
                    info!("picked a window");
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
                    info!("clicked on nothing. sending background debug event");
                    tx_out.send(OutgoingMessage {
                        recipient: Default::default(),
                        command: APICommand::Debug(DebugMessage::BackgroundReceivedMouseEvent)
                    }).unwrap();
                }
            }
            APICommand::KeyDown(evt) => {
                match evt.key {
                    KeyCode::ESC => {
                        tx_out.send(OutgoingMessage{
                            recipient:Default::default(),
                            command: APICommand::Debug(DebugMessage::RequestServerShutdown)
                        }).unwrap();
                        thread::sleep(Duration::from_millis(500));
                        return false;
                    }
                    _ => {
                        info!("got a key down event");
                        if let Some(id) = state.get_focused_window() {
                            if let Some(win) = state.lookup_window(*id) {
                                let wid = win.id.clone();
                                let aid = win.owner.clone();
                                tx_out.send(OutgoingMessage {
                                    recipient: aid,
                                    command: APICommand::KeyDown(KeyDownEvent {
                                        app_id: aid,
                                        window_id: wid,
                                        original_timestamp: evt.original_timestamp,
                                        key: evt.key
                                    })
                                }).unwrap();
                            }
                        }
                    }
                }
            }
            APICommand::Debug(DebugMessage::ScreenCapture(rect, str)) => {
                let pth = PathBuf::from("./screencapture.png");
                info!("rect for screen capture {:?}",pth);
                // export_to_png(&buf, &pth);
                tx_out.send(OutgoingMessage {
                    recipient: Default::default(),
                    command: APICommand::Debug(DebugMessage::ScreenCaptureResponse()),
                }).unwrap();
            }
            APICommand::WMConnectResponse(res) => {
                // pt("the central said hi back");
            }
            _ => {
                info!("unhandled message {:?}", cmd);
            }
        };
    }


    {
        //redraw all the windows
        plat.clear();
        // surf.buf.clear(&BLACK);
        for win in state.window_list() {
            // let (wc, tc) = if state.is_focused_window(win) {
            //     (FOCUSED_WINDOW_COLOR, FOCUSED_TITLEBAR_COLOR)
            // } else {
            //     (WINDOW_COLOR, TITLEBAR_COLOR)
            // };
            // surf.buf.draw_rect(win.external_bounds(), wc,WINDOW_BORDER_WIDTH);
            // plat.draw_rect(win.external_bounds(), &wc, WINDOW_BORDER_WIDTH);
            // plat.fill_rect(win.titlebar_bounds(), &tc);
            let bd = win.content_bounds();
            let MAGENTA = ARGBColor::new_rgb(255, 0, 255);
            plat.fill_rect(bd, &MAGENTA);
            plat.draw_image(win.content_bounds().x, win.content_bounds().y, &win.backbuffer);
            // surf.copy_from(bd.x, bd.y, &win.backbuffer)
        }
    }

    plat.service_loop();
    true
}
