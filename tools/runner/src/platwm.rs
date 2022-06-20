use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::{JoinHandle, spawn};
use std::time::{Duration, Instant};
use log::info;
use serde::Deserialize;
use common::{APICommand, ARGBColor, BLACK, DebugMessage, HelloWindowManager, IncomingMessage, Point, Rect, WHITE, WINDOW_MANAGER_PORT, WindowResized};
use common::events::{KeyCode, KeyDownEvent, MouseButton, MouseDownEvent, MouseUpEvent};
use common::font::{FontInfo2, load_font_from_json};
use common::graphics::{draw_test_pattern, GFXBuffer, PixelLayout};
use common_wm::{AppMouseGesture, FOCUSED_TITLEBAR_COLOR, FOCUSED_WINDOW_COLOR, InputGesture, NoOpGesture, OutgoingMessage, TITLEBAR_COLOR, WINDOW_BORDER_WIDTH, WINDOW_COLOR, WindowDragGesture, WindowManagerState, WindowResizeGesture};
use plat::{make_plat, Plat};

pub struct PlatformWindowManager {
    pub stream: TcpStream,
    pub plat: Plat,
    pub state: WindowManagerState,
    pub receiving_handle: JoinHandle<()>,
    pub sending_handle: JoinHandle<()>,
    pub tx_out: Sender<OutgoingMessage>,
    pub rx_in: Receiver<IncomingMessage>,
    pub background: GFXBuffer,
    pub font: FontInfo2,
    pub gesture: Box<dyn InputGesture>,
    pub cursor: Point,
    pub cursor_image: GFXBuffer,
    tick:u128,
    fps:Vec<u128>,
}

impl PlatformWindowManager {
    pub(crate) fn shutdown(&mut self) {
        self.plat.shutdown()
    }
}

impl PlatformWindowManager {
    pub(crate) fn init(w: i32, h: i32) -> Option<PlatformWindowManager> {
        let conn_string = format!("localhost:{}", WINDOW_MANAGER_PORT);

        match TcpStream::connect(conn_string) {
            Ok(stream) => {
                info!("connected to the central server");

                let (tx_out, rx_out) = mpsc::channel::<OutgoingMessage>();
                let (tx_in, mut rx_in) = mpsc::channel::<IncomingMessage>();
                let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

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
                                            println!("err {}", e);
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

                let mut plat = make_plat(stop.clone(), tx_in.clone()).unwrap();
                let bds = plat.get_screen_bounds();
                let mut background = GFXBuffer::new(bds.w as u32, bds.h as u32, &plat.get_preferred_pixel_layout());
                plat.register_image2(&background);
                let mut cursor_image:GFXBuffer = GFXBuffer::from_png_file("../../resources/cursor.png").to_layout(plat.get_preferred_pixel_layout());
                plat.register_image2(&cursor_image);
                let font = load_font_from_json("../../resources/default-font.json").unwrap();
                Some(PlatformWindowManager {
                    stream,
                    state: WindowManagerState::init(plat.get_preferred_pixel_layout()),
                    plat,
                    sending_handle,
                    receiving_handle,
                    tx_out,
                    rx_in,
                    background,
                    font,
                    gesture: Box::new(NoOpGesture::init()) as Box<dyn InputGesture>,
                    cursor:  Point::init(0,0),
                    cursor_image,
                    tick: 0,
                    fps: vec![],
                })
            }
            _ => {
                info!("could not connect to server at");
                None
            }
        }
    }


    pub fn main_service_loop(&mut self) -> bool {
        let start = Instant::now();
            // println!("Native WM service loop");
        self.plat.service_input();
        let cont = self.process_input();
        if !cont {
            return false
        }

        // check for windows that need to be resized
        self.check_window_sizes();



        self.draw_screen();
        self.plat.service_loop();
        self.fps.push(start.elapsed().as_millis());

        if self.fps.len() > 60 {
            self.fps.remove(0);
        }
        if self.tick % 30 == 0 {
            let mut total = 0;
            for dur in &self.fps {
                total += dur;
            }
            // println!("calculated avg frame time {}", (total as f64)/(self.fps.len() as f64));
        }
        self.tick += 1;
        //thread::sleep(Duration::from_millis(100));
        true
    }

    fn process_input(&mut self) -> bool {
        for cmd in self.rx_in.try_iter() {
            // pt(&format!("received {:?}", cmd));
            match cmd.command {
                APICommand::SystemShutdown => {
                    info!("the core is shutting down. bye");
                    return false;
                }
                APICommand::AppConnectResponse(res) => {
                    self.state.add_app(res.app_id);
                },
                APICommand::AppDisconnected(dis) => {
                    info!("app disconnected. removing windows");
                    if let Some(app) = &self.state.find_app(dis.app_id) {
                        for win in &app.windows {
                            self.plat.unregister_image2(&win.backbuffer);
                        }
                    }
                    self.state.remove_app(dis.app_id);
                }
                APICommand::OpenWindowResponse(ow) => {
                    let win_id = self.state.add_window(ow.app_id, ow.window_id, &ow.bounds);
                    if let Some(win) = self.state.lookup_window(win_id) {
                        self.plat.register_image2(&win.backbuffer);
                    }
                },
                APICommand::DrawRectCommand(dr) => {
                    if let Some(mut win) = self.state.lookup_window(dr.window_id) {
                        // info!("NativeWM: draw rect to window {:?} {:?}", &dr.rect, &dr.color);
                        win.backbuffer.fill_rect(dr.rect, &dr.color);
                        // buf.copy_from(win.position.x, win.position.y, &win.backbuffer);
                    }
                },
                APICommand::DrawImageCommand(dr) => {
                    if let Some(mut win) = self.state.lookup_window(dr.window_id) {
                        // info!("NativeWM: draw image to window {:?}", &dr.rect);
                        win.backbuffer.fill_rect_with_image(&dr.rect,&dr.buffer);
                    }
                },
                APICommand::MouseUp(evt) => {
                    self.gesture.mouse_up(evt, &mut self.state, &self.tx_out);
                    self.gesture = Box::new(NoOpGesture::init()) as Box<dyn InputGesture>;
                },
                APICommand::MouseMove(evt) => {
                    self.cursor = Point::init(evt.x, evt.y);
                    self.gesture.mouse_move(evt, &mut self.state, &self.tx_out);
                }
                APICommand::MouseDown(evt) => {
                    let point = Point::init(evt.x, evt.y);
                    if let Some(win) = self.state.pick_window_at(point) {
                        info!("picked a window");
                        let wid = win.id.clone();
                        let aid = win.owner.clone();

                        if win.titlebar_bounds().contains(&point) {
                            info!("inside the titlebar");
                            self.gesture = Box::new(WindowDragGesture::init(point, win.id));
                            self.gesture.mouse_down(evt, &mut self.state, &self.tx_out);
                        } else if win.resize_bounds().contains(&point) {
                            info!("inside the resize control");
                            self.gesture = Box::new(WindowResizeGesture::init(point, win.id));
                            self.gesture.mouse_down(evt, &mut self.state, &self.tx_out);
                        } else {
                            self.gesture = Box::new(AppMouseGesture::init(aid,win.id));
                            self.gesture.mouse_down(evt,&mut self.state, &self.tx_out);
                        }
                        self.state.set_focused_window(wid);
                        self.tx_out.send(OutgoingMessage {
                            recipient: Default::default(),
                            command: APICommand::Debug(DebugMessage::WindowFocusChanged(String::from("foo")))
                        }).unwrap();
                    } else {
                        info!("clicked on nothing. sending background debug event");
                        self.tx_out.send(OutgoingMessage {
                            recipient: Default::default(),
                            command: APICommand::Debug(DebugMessage::BackgroundReceivedMouseEvent)
                        }).unwrap();
                    }
                }
                APICommand::KeyDown(evt) => {
                    match evt.key {
                        KeyCode::ESC => {
                            self.tx_out.send(OutgoingMessage {
                                recipient: Default::default(),
                                command: APICommand::Debug(DebugMessage::RequestServerShutdown)
                            }).unwrap();
                            thread::sleep(Duration::from_millis(500));
                            return false;
                        }
                        _ => {
                            info!("got a key down event");
                            if let Some(id) = self.state.get_focused_window() {
                                if let Some(win) = self.state.lookup_window(*id) {
                                    let wid = win.id.clone();
                                    let aid = win.owner.clone();
                                    self.tx_out.send(OutgoingMessage {
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
                    self.tx_out.send(OutgoingMessage {
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
        return true
    }
    fn check_window_sizes(&mut self) {
        for win in self.state.window_list_mut() {
            // println!("buffer bounds {} {}",win.backbuffer.bounds(), win.content_bounds());
            if (!win.backbuffer.bounds().size().eq(&win.content_bounds().size())) {
                println!("not equal");
                info!("NativeWM: resizing window backbuffer to {}", &win.content_bounds().size());
                // win.backbuffer.fill_rect_with_image(&dr.rect,&dr.buffer);
                self.plat.unregister_image2(&win.backbuffer);
                win.backbuffer = GFXBuffer::new(win.content_size.w as u32, win.content_size.h as u32, &win.backbuffer.layout);
                self.plat.register_image2(&win.backbuffer);
                self.tx_out.send(OutgoingMessage {
                    recipient: win.owner,
                    command: APICommand::WindowResized(WindowResized {
                        app_id: win.owner,
                        window_id: win.id,
                        size: win.content_size,
                    })
                }).unwrap();

            }
        }
    }
    fn draw_screen(&mut self) {
            self.plat.clear();

            self.background.clear(&ARGBColor::new_rgb(120,128,128));
            self.background.fill_rect(Rect::from_ints(0,0,25,25), &ARGBColor::new_rgb(0,0,0));
            self.background.fill_rect(Rect::from_ints(0,25,25,25), &ARGBColor::new_rgb(255,0,0));
            self.background.fill_rect(Rect::from_ints(0,50,25,25), &ARGBColor::new_rgb(0,255,0));
            self.background.fill_rect(Rect::from_ints(0,75,25,25), &ARGBColor::new_rgb(0,0,255));
            self.background.fill_rect(Rect::from_ints(0,100,25,25), &ARGBColor::new_rgb(255,255,0));
            self.background.fill_rect(Rect::from_ints(0,125,25,25), &ARGBColor::new_rgb(0,255,255));
            self.background.fill_rect(Rect::from_ints(0,150,25,25), &ARGBColor::new_rgb(255,0,255));
            self.background.fill_rect(Rect::from_ints(0,175,25,25), &ARGBColor::new_rgb(255,255,255));

            // draw_test_pattern(&mut self.background);
            // self.font.draw_text_at(&mut self.background,"Greetings Earthling",40,40,&ARGBColor::new_rgb(0,255,0));
            self.plat.draw_image(&Point::init(0, 0), &self.background.bounds(), &self.background);
            // self.background.to_png(&PathBuf::from("output.png"));
            // panic!();
            for win in self.state.window_list() {
                let (wc, tc) = if self.state.is_focused_window(win) {
                    (FOCUSED_WINDOW_COLOR, FOCUSED_TITLEBAR_COLOR)
                } else {
                    (WINDOW_COLOR, TITLEBAR_COLOR)
                };
                // self.plat.draw_rect(win.external_bounds(), &wc, WINDOW_BORDER_WIDTH);
                // draw the titlebar
                self.plat.fill_rect(win.titlebar_bounds(), &tc);
                //draw the content
                let bd = win.content_bounds();
                let MAGENTA = ARGBColor::new_rgb(255, 0, 255);
                self.plat.fill_rect(bd, &MAGENTA);
                self.plat.draw_image(&win.content_bounds().position(), &win.backbuffer.bounds(), &win.backbuffer);
                // draw the resize button
                self.plat.fill_rect(win.resize_bounds(), &MAGENTA);
            }
            // draw the cursor
            self.plat.draw_image(&self.cursor,&self.cursor_image.bounds(),&self.cursor_image);

    }
}
