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
use common::{APICommand, ARGBColor, BLACK, DebugMessage, HelloWindowManager, IncomingMessage, Point, Rect, WHITE, WINDOW_MANAGER_PORT};
use common::events::{KeyCode, KeyDownEvent, MouseButton, MouseDownEvent};
use common::font::{FontInfo2, load_font_from_json};
use common::graphics::{ColorDepth, export_to_png, GFXBuffer, PixelLayout};
use common_wm::{FOCUSED_TITLEBAR_COLOR, FOCUSED_WINDOW_COLOR, InputGesture, NoOpGesture, OutgoingMessage, TITLEBAR_COLOR, WINDOW_BORDER_WIDTH, WINDOW_COLOR, WindowDragGesture, WindowManagerState};
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
                let mut background = GFXBuffer::new(ColorDepth::CD32(), bds.w as u32, bds.h as u32, PixelLayout::RGBA());
                background.fast = true;
                plat.register_image2(&background);
                let mut cursor_image:GFXBuffer = GFXBuffer::from_png_file("../../resources/cursor.png");
                plat.register_image2(&cursor_image);
                cursor_image.fast = true;
                let font = load_font_from_json("../../resources/default-font.json").unwrap();
                Some(PlatformWindowManager {
                    stream,
                    state: WindowManagerState::init(),
                    plat,
                    sending_handle,
                    receiving_handle,
                    tx_out,
                    rx_in,
                    background,
                    font,
                    gesture: Box::new(NoOpGesture::init()) as Box<dyn InputGesture>,
                    cursor:Point::init(0,0),
                    cursor_image,
                })
            }
            _ => {
                info!("could not connect to server at");
                None
            }
        }
    }


    pub fn main_service_loop(&mut self) -> bool {
        // println!("Native WM service loop");
        self.plat.service_input();
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
                        info!("NativeWM: draw rect to window {:?} {:?}", &dr.rect, &dr.color);
                        win.backbuffer.fill_rect(dr.rect, &dr.color);
                        // buf.copy_from(win.position.x, win.position.y, &win.backbuffer);
                    }
                },
                APICommand::DrawImageCommand(dr) => {
                    if let Some(mut win) = self.state.lookup_window(dr.window_id) {
                        info!("NativeWM: draw image to window {:?}", &dr.rect);
                        win.backbuffer.fill_rect_with_image(&dr.rect,&dr.buffer);
                    }
                },
                APICommand::MouseUp(evt) => {
                    self.gesture = Box::new(NoOpGesture::init()) as Box<dyn InputGesture>;
                },
                APICommand::MouseMove(evt) => {
                    self.cursor = Point::init(evt.x, evt.y);
                    self.gesture.mouse_move(evt, &mut self.state);
                }
                APICommand::MouseDown(evt) => {
                    let point = Point::init(evt.x, evt.y);
                    if let Some(win) = self.state.pick_window_at(point) {
                        info!("picked a window");
                        let wid = win.id.clone();
                        let aid = win.owner.clone();

                        if win.titlebar_bounds().contains(point) {
                            info!("inside the titlebar");
                            self.gesture = Box::new(WindowDragGesture::init(point,win.id));
                            self.gesture.mouse_down(evt,&mut self.state);
                        } else {
                            let app_point = point.subtract(win.content_bounds().position());
                            self.tx_out.send(OutgoingMessage {
                                recipient: aid,
                                command: APICommand::MouseDown(MouseDownEvent {
                                    app_id: aid,
                                    window_id: wid,
                                    original_timestamp: evt.original_timestamp,
                                    button: MouseButton::Primary,
                                    x: app_point.x,
                                    y: app_point.y
                                })
                            }).unwrap();
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


        {
            self.plat.clear();

            // self.background.clear(&ARGBColor::new_rgb(100,100,100));
            // self.background.fill_rect(Rect::from_ints(0,0,25,25), &BLACK);
            // self.font.draw_text_at(&mut self.background,"Greetings Earthling",40,40,&ARGBColor::new_rgb(0,255,0));
            // self.plat.draw_image(0, 0, &self.background);
            for win in self.state.window_list() {
                let (wc, tc) = if self.state.is_focused_window(win) {
                    (FOCUSED_WINDOW_COLOR, FOCUSED_TITLEBAR_COLOR)
                } else {
                    (WINDOW_COLOR, TITLEBAR_COLOR)
                };
                self.plat.draw_rect(win.external_bounds(), &wc, WINDOW_BORDER_WIDTH);
                self.plat.fill_rect(win.titlebar_bounds(), &tc);
                let bd = win.content_bounds();
                let MAGENTA = ARGBColor::new_rgb(255, 0, 255);
                self.plat.fill_rect(bd, &MAGENTA);
                self.plat.draw_image(win.content_bounds().x, win.content_bounds().y, &win.backbuffer);
            }
            self.plat.draw_image(self.cursor.x,self.cursor.y,&self.cursor_image);
        }

        self.plat.service_loop();
        true
    }
}


#[test]
fn buffer_clear_CD32_RGBA_speed() {
    let start = Instant::now();
    let w = 1024;
    let h = 1024;
    let color = ARGBColor::new_rgb(100,100,100);
    let mut background = GFXBuffer::new(ColorDepth::CD32(), w, h, PixelLayout::ARGB());
    background.fast = true;
    for _ in 0..10 {
        background.clear(&color);
    }
    // 2.55s vs 0.76s  ~= 3.3x faster
    println!("took {}",start.elapsed().as_secs_f32());
}

#[test]
fn buffer_clear_CD24_RGB_speed() {
    let start = Instant::now();
    let w = 1024;
    let h = 1024;
    let color = ARGBColor::new_rgb(100,100,100);
    let mut background = GFXBuffer::new(ColorDepth::CD24(), w, h, PixelLayout::RGB());
    background.fast = true;
    for n in 0..10 {
        background.clear(&color);
    }
    // 2.55s vs 0.76s  ~= 3.3x faster
    println!("took {}",start.elapsed().as_secs_f32());
}


#[test]
fn buffer_clear_CD15_RGB565_speed() {
    let start = Instant::now();
    let w = 1024;
    let h = 1024;
    let color = ARGBColor::new_rgb(100,100,100);
    let mut background = GFXBuffer::new(ColorDepth::CD16(), w, h, PixelLayout::RGB565());
    background.fast = true;
    for n in 0..10 {
        background.clear(&color);
    }
    // 2.55s vs 0.76s  ~= 3.3x faster
    println!("took {}",start.elapsed().as_secs_f32());
}


#[test]
fn buffer_fill_rect_cd32_rgba_speed() {
    let w = 1024;
    let h = 1024;
    let color = ARGBColor::new_rgb(100,100,100);

    let types = [
        (ColorDepth::CD32(),PixelLayout::ARGB()),
        (ColorDepth::CD32(),PixelLayout::RGBA()),
        (ColorDepth::CD24(),PixelLayout::RGB())];

    for (depth,layout) in types {
        let mut background = GFXBuffer::new(depth, w, h, layout);
        background.fast = true;
        background.clear(&BLACK);
        let start = Instant::now();
        let bounds = Rect::from_ints(500, 500, 1000, 1000);
        for _ in 0..10 {
            background.fill_rect(bounds, &color);
        }
        println!("took {}",start.elapsed().as_secs_f32());
        // println!("is black {:?}",background.get_pixel_vec(&PixelLayout::RGBA(),0,0));
        // println!("is color {:?}",background.get_pixel_vec(&PixelLayout::RGBA(),600,600));
        assert_eq!(background.get_pixel_vec(&PixelLayout::RGBA(),0,0),BLACK.as_layout(&PixelLayout::RGBA()));
        assert_eq!(background.get_pixel_vec(&PixelLayout::RGBA(),600,600),color.as_layout(&PixelLayout::RGBA()));
    }

    // 2.55s vs 0.76s  ~= 3.3x faster

}
