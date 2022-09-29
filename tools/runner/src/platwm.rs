use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::{JoinHandle, spawn};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use log::info;
use serde::Deserialize;
use uuid::Uuid;
use common::{APICommand, DebugMessage, IncomingMessage, WINDOW_MANAGER_PORT, WindowResized};
use common::events::{KeyDownEvent};
use common::generated::KeyCode;
use common_wm::{AppMouseGesture, CentralConnection, FOCUSED_TITLEBAR_COLOR, FOCUSED_WINDOW_COLOR, InputGesture, NoOpGesture, start_wm_network_connection, TITLE_BAR_HEIGHT, TITLEBAR_COLOR, Window, WINDOW_BUTTON_COLOR, WINDOW_COLOR, WindowCloseButtonGesture, WindowDragGesture, WindowManagerState, WindowResizeGesture};
use gfx::font::{FontInfo2, load_font_from_json};
use gfx::graphics::{ARGBColor, BLACK, GFXBuffer, Point, Rect, WHITE};
use minibuf::{make_plat, Plat};

pub struct PlatformWindowManager {
    pub connection:CentralConnection,
    pub plat: Plat,
    pub state: WindowManagerState,
    pub rx_in: Receiver<IncomingMessage>,
    pub background: GFXBuffer,
    pub font: FontInfo2,
    pub gesture: Box<dyn InputGesture>,
    pub cursor: Point,
    pub cursor_image: GFXBuffer,
    pub debug_pos: Point,
    pub debug_buffer: GFXBuffer,
    pub title_buffer: GFXBuffer,
    pub exit_button_bounds:Rect,

    tick:u128,
    fps:Vec<u128>,
}

impl PlatformWindowManager {
    pub(crate) fn make_fake_window(&mut self, title: &String, bounds: &Rect) {
        let fake_app = Uuid::new_v4();
        self.state.add_app(fake_app);
        let fake_window = Uuid::new_v4();
        self.state.add_window(fake_app, fake_window, bounds, title);
        if let Some(win) = self.state.lookup_window(fake_window) {
            self.plat.register_image2(&win.backbuffer);
        }
    }
}

impl PlatformWindowManager {
    pub(crate) fn shutdown(&mut self) {
        self.plat.shutdown()
    }
}

impl PlatformWindowManager {
    pub(crate) fn init(w: u32, h: u32, scale:u32) -> Option<PlatformWindowManager> {
        let conn_string = format!("localhost:{}", WINDOW_MANAGER_PORT);
        let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let (tx_in, rx_in) = mpsc::channel::<IncomingMessage>();
        if let Some(central) = start_wm_network_connection(stop.clone(), tx_in.clone()) {
            let mut plat = make_plat(stop.clone(), tx_in, w, h, scale).unwrap();
            let bds = plat.get_screen_bounds();
            let background = GFXBuffer::new(bds.w as u32, bds.h as u32, &plat.get_preferred_pixel_layout());
            plat.register_image2(&background);
            let cursor_image: GFXBuffer = GFXBuffer::from_png_file("./resources/cursor.png").to_layout(plat.get_preferred_pixel_layout());
            plat.register_image2(&cursor_image);
            let font = load_font_from_json("./resources/default-font.json").unwrap();
            let debug_buffer = GFXBuffer::new(200, 50, &plat.get_preferred_pixel_layout());
            plat.register_image2(&debug_buffer);
            let title_buffer = GFXBuffer::new(200, TITLE_BAR_HEIGHT as u32, &plat.get_preferred_pixel_layout());
            plat.register_image2(&title_buffer);
            Some(PlatformWindowManager {
                connection:central,
                state: WindowManagerState::init(plat.get_preferred_pixel_layout()),
                plat,
                rx_in,
                background,
                font,
                gesture: Box::new(NoOpGesture::init()) as Box<dyn InputGesture>,
                cursor: Point::init(0, 0),
                cursor_image,
                tick: 0,
                fps: vec![],
                exit_button_bounds: Rect::from_ints(bds.w - 40, bds.h - 20, 40, 20),
                debug_pos: Point::init(0, bds.h - 50),
                debug_buffer,
                title_buffer,
            })
        } else {
            info!("could not connect to server at");
            None
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
        self.tick += 1;
        true
    }

    fn process_input(&mut self) -> bool {
        for cmd in self.rx_in.try_iter() {
            if cmd.trace {
                info!("platwm Lib received {:?}", cmd);
            }
            let src = cmd.clone();
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
                    let win_id = self.state.add_window(ow.app_id, ow.window_id, &ow.bounds, &ow.window_title);
                    if let Some(win) = self.state.lookup_window(win_id) {
                        self.plat.register_image2(&win.backbuffer);
                    }
                },
                APICommand::DrawRectCommand(dr) => {
                    if let Some(win) = self.state.lookup_window_mut(dr.window_id) {
                        // info!("NativeWM: draw rect to window {:?} {:?}", &dr.rect, &dr.color);
                        win.backbuffer.fill_rect(&dr.rect, &dr.color);
                        // buf.copy_from(win.position.x, win.position.y, &win.backbuffer);
                    }
                },
                APICommand::DrawImageCommand(dr) => {
                    if let Some(win) = self.state.lookup_window_mut(dr.window_id) {
                        // info!("NativeWM: draw image to window {:?}", &dr.rect);
                        win.backbuffer.fill_rect_with_image(&dr.rect,&dr.buffer);
                    }
                },
                APICommand::MouseUp(evt) => {
                    self.gesture.mouse_up(evt, &mut self.state, &self.connection.tx_out);
                    self.gesture = Box::new(NoOpGesture::init()) as Box<dyn InputGesture>;
                },
                APICommand::MouseMove(evt) => {
                    self.cursor = Point::init(evt.x, evt.y);
                    self.gesture.mouse_move(evt, &mut self.state, &self.connection.tx_out);
                }
                APICommand::MouseDown(evt) => {
                    let point = Point::init(evt.x, evt.y);
                    // info!("checking mouse down path");
                    if self.exit_button_bounds.contains(&point) {
                        info!("clicked the exit button!");
                        self.connection.tx_out.send(IncomingMessage {
                            source:Default::default(),
                            trace: cmd.trace,
                            timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                            command: APICommand::Debug(DebugMessage::RequestServerShutdown)
                        }).unwrap();
                        thread::sleep(Duration::from_millis(500));
                        return false;
                    }
                    if let Some(win) = self.state.pick_window_at(point) {
                        // info!("picked a window");
                        let wid = win.id.clone();
                        let aid = win.owner.clone();

                        if win.close_button_bounds().contains(&point) {
                            info!("inside the close button");
                            self.gesture = Box::new(WindowCloseButtonGesture::init(point, win.id));
                            self.gesture.mouse_down(evt,&src, &mut self.state, &self.connection.tx_out);
                        } else if win.titlebar_bounds().contains(&point) {
                            info!("inside the title bar");
                            self.gesture = Box::new(WindowDragGesture::init(point, win.id));
                            self.gesture.mouse_down(evt, &src,&mut self.state, &self.connection.tx_out);
                        } else if win.resize_bounds().contains(&point) {
                            info!("inside the resize control");
                            self.gesture = Box::new(WindowResizeGesture::init(point, win.id));
                            self.gesture.mouse_down(evt, &src,&mut self.state, &self.connection.tx_out);
                        } else {
                            // it needs to go to the app
                            // info!("inside the window content for the app");
                            self.gesture = Box::new(AppMouseGesture::init(aid,win.id));
                            self.gesture.mouse_down(evt, &src,&mut self.state, &self.connection.tx_out);
                        }
                        self.state.set_focused_window(wid);
                        self.state.raise_window(wid);
                        // info!("sending out focus changed");
                        self.connection.tx_out.send(IncomingMessage {
                            source:Default::default(),
                            trace: cmd.trace,
                            timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                            // recipient: Default::default(),
                            command: APICommand::Debug(DebugMessage::WindowFocusChanged(String::from("foo")))
                        }).unwrap();
                    } else {
                        // info!("clicked on nothing. sending background debug event");
                        self.connection.tx_out.send(IncomingMessage {
                            source:Default::default(),
                            trace: cmd.trace,
                            timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                            // recipient: Default::default(),
                            command: APICommand::Debug(DebugMessage::BackgroundReceivedMouseEvent)
                        }).unwrap();
                    }
                }
                APICommand::KeyDown(evt) => {
                    match evt.code {
                        KeyCode::ESCAPE => {
                            self.connection.tx_out.send(IncomingMessage {
                                source:Default::default(),
                                trace: false,
                                timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                                // recipient: Default::default(),
                                command: APICommand::Debug(DebugMessage::RequestServerShutdown)
                            }).unwrap();
                            thread::sleep(Duration::from_millis(500));
                            return false;
                        }
                        _ => {
                            // info!("got a key down event {:?}",evt);
                            if let Some(id) = self.state.get_focused_window() {
                                if let Some(win) = self.state.lookup_window(*id) {
                                    let wid = win.id.clone();
                                    let aid = win.owner.clone();
                                    self.connection.tx_out.send(IncomingMessage {
                                        source:Default::default(),
                                        trace: false,
                                        timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                                        // recipient: aid,
                                        command: APICommand::KeyDown(KeyDownEvent {
                                            app_id: aid,
                                            window_id: wid,
                                            original_timestamp: evt.original_timestamp,
                                            key: evt.key,
                                            code: evt.code,
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
                    self.connection.tx_out.send(IncomingMessage {
                        source:Default::default(),
                        trace: false,
                        timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                        // recipient: Default::default(),
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
        true
    }
    fn check_window_sizes(&mut self) {
        for win in self.state.window_list_mut() {
            // println!("buffer bounds {} {}",win.backbuffer.bounds(), win.content_bounds());
            if !win.backbuffer.bounds().size().eq(&win.content_bounds().size()) {
                println!("not equal");
                info!("NativeWM: resizing window backbuffer to {}", &win.content_bounds().size());
                // win.backbuffer.fill_rect_with_image(&dr.rect,&dr.buffer);
                self.plat.unregister_image2(&win.backbuffer);
                win.backbuffer = GFXBuffer::new(win.content_size.w as u32, win.content_size.h as u32, &win.backbuffer.layout);
                self.plat.register_image2(&win.backbuffer);
                self.connection.tx_out.send(IncomingMessage {
                    source:Default::default(),
                    trace: false,
                    timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                    // recipient: win.owner,
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
        self.plat.draw_image(&Point::init(0, 0), &self.background.bounds(), &self.background);
        let MAGENTA:ARGBColor = ARGBColor::new_rgb(255, 0, 255);
        self.draw_windows();

        if let Some(rect) =  self.state.resize_rect {
            let size = 2;
            let left = Rect::from_ints(rect.x,rect.y,size,rect.h);
            self.plat.fill_rect(left,&WHITE);
            let right = Rect::from_ints(rect.x+rect.w-size, rect.y, size, rect.h);
            self.plat.fill_rect(right, &WHITE);
            let top = Rect::from_ints(rect.x, rect.y, rect.w, size);
            self.plat.fill_rect(top, &WHITE);
            let bot = Rect::from_ints(rect.x, rect.y+rect.h-size, rect.w, size);
            self.plat.fill_rect(bot, &WHITE);
        }

        //draw the fps amount
        let mut total = 0;
        for dur in &self.fps {
            total += dur;
        }
        let avg_frame_length = (total as f64)/(self.fps.len() as f64);
        self.debug_buffer.clear(&BLACK);
        // println!("avg frame: {:.2}",avg_frame_length);
        self.font.draw_text_at(&mut self.debug_buffer,
                               &format!("avg frame: {:.2}", avg_frame_length),
                               3, 20, &WHITE);
        self.plat.draw_image(&self.debug_pos, &self.debug_buffer.bounds(), &self.debug_buffer);

        //draw the exit button
        self.plat.fill_rect(self.exit_button_bounds, &MAGENTA);

        // draw the cursor
        self.plat.draw_image(&self.cursor,&self.cursor_image.bounds(),&self.cursor_image);
    }

    fn draw_windows(&mut self) {
        let magenta:ARGBColor = ARGBColor::new_rgb(255, 0, 255);
        let wins:Vec<&Window> = self.state.get_windows_in_order();
        for win in wins {
            let tc = if self.state.is_focused_window(win) {
                FOCUSED_TITLEBAR_COLOR
            } else {
                TITLEBAR_COLOR
            };
            //draw window contents
            self.plat.draw_image(&win.content_bounds().position(), &win.backbuffer.bounds(), &win.backbuffer);

            // draw the titlebar
            self.plat.fill_rect(win.titlebar_bounds(), &tc);

            let gray = ARGBColor::new_argb(0, 100, 100, 100);
            // draw text to a scratch buffer
            &self.title_buffer.clear(&tc);
            self.font.draw_text_at(&mut self.title_buffer, &win.title, win.close_button_bounds().w+2, 7, &BLACK);
            let glyph = 14; // close glyph
            self.font.draw_glyph_at(&mut self.title_buffer, glyph,5,5,&BLACK );

            let pt = win.titlebar_bounds().position();
            let tw = win.titlebar_bounds().w;
            let sub_bounds = Rect::from_ints(0, 0, tw.min(self.title_buffer.width as i32), self.title_buffer.height as i32);
            self.plat.draw_image(&pt,&sub_bounds,&self.title_buffer);
            // draw the resize button
            self.plat.fill_rect(win.resize_bounds(), &magenta);
        }
    }
}
