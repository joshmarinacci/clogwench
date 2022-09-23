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
use common::{APICommand, ARGBColor, BLACK, DebugMessage, IncomingMessage, Point, Rect, WHITE, WINDOW_MANAGER_PORT, WindowResized};
use common::events::{KeyDownEvent};
use common::font::{FontInfo2, load_font_from_json};
use common::generated::KeyCode;
use common::graphics::{GFXBuffer};
use common_wm::{AppMouseGesture, CentralConnection, FOCUSED_TITLEBAR_COLOR, FOCUSED_WINDOW_COLOR, InputGesture, NoOpGesture, OutgoingMessage, start_wm_network_connection, TITLEBAR_COLOR, WINDOW_BUTTON_COLOR, WINDOW_COLOR, WindowCloseButtonGesture, WindowDragGesture, WindowManagerState, WindowResizeGesture};
use plat::{make_plat, Plat};

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
    pub exit_button_bounds:Rect,

    tick:u128,
    fps:Vec<u128>,
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
            let cursor_image: GFXBuffer = GFXBuffer::from_png_file("../../resources/cursor.png").to_layout(plat.get_preferred_pixel_layout());
            plat.register_image2(&cursor_image);
            let font = load_font_from_json("../../resources/default-font.json").unwrap();
            let fps_buff = GFXBuffer::new(200, 50, &plat.get_preferred_pixel_layout());
            plat.register_image2(&fps_buff);
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
                debug_buffer: fps_buff,
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
                    let win_id = self.state.add_window(ow.app_id, ow.window_id, &ow.bounds, &ow.window_title);
                    if let Some(win) = self.state.lookup_window(win_id) {
                        self.plat.register_image2(&win.backbuffer);
                    }
                },
                APICommand::DrawRectCommand(dr) => {
                    if let Some(win) = self.state.lookup_window_mut(dr.window_id) {
                        // info!("NativeWM: draw rect to window {:?} {:?}", &dr.rect, &dr.color);
                        win.backbuffer.fill_rect(dr.rect, &dr.color);
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
                    if self.exit_button_bounds.contains(&point) {
                        info!("clicked the exit button!");
                        self.connection.tx_out.send(OutgoingMessage {
                            recipient: Default::default(),
                            command: APICommand::Debug(DebugMessage::RequestServerShutdown)
                        }).unwrap();
                        thread::sleep(Duration::from_millis(500));
                        return false;
                    }
                    if let Some(win) = self.state.pick_window_at(point) {
                        info!("picked a window");
                        let wid = win.id.clone();
                        let aid = win.owner.clone();

                        if win.close_button_bounds().contains(&point) {
                            // info!("inside the close button");
                            self.gesture = Box::new(WindowCloseButtonGesture::init(point, win.id));
                            self.gesture.mouse_down(evt, &mut self.state, &self.connection.tx_out);
                        } else if win.titlebar_bounds().contains(&point) {
                            // info!("inside the titlebar");
                            self.gesture = Box::new(WindowDragGesture::init(point, win.id));
                            self.gesture.mouse_down(evt, &mut self.state, &self.connection.tx_out);
                        } else if win.resize_bounds().contains(&point) {
                            // info!("inside the resize control");
                            self.gesture = Box::new(WindowResizeGesture::init(point, win.id));
                            self.gesture.mouse_down(evt, &mut self.state, &self.connection.tx_out);
                        } else {
                            self.gesture = Box::new(AppMouseGesture::init(aid,win.id));
                            self.gesture.mouse_down(evt,&mut self.state, &self.connection.tx_out);
                        }
                        self.state.set_focused_window(wid);
                        self.state.raise_window(wid);
                        self.connection.tx_out.send(OutgoingMessage {
                            recipient: Default::default(),
                            command: APICommand::Debug(DebugMessage::WindowFocusChanged(String::from("foo")))
                        }).unwrap();
                    } else {
                        info!("clicked on nothing. sending background debug event");
                        self.connection.tx_out.send(OutgoingMessage {
                            recipient: Default::default(),
                            command: APICommand::Debug(DebugMessage::BackgroundReceivedMouseEvent)
                        }).unwrap();
                    }
                }
                APICommand::KeyDown(evt) => {
                    match evt.code {
                        KeyCode::ESCAPE => {
                            self.connection.tx_out.send(OutgoingMessage {
                                recipient: Default::default(),
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
                                    self.connection.tx_out.send(OutgoingMessage {
                                        recipient: aid,
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
                    self.connection.tx_out.send(OutgoingMessage {
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
                self.connection.tx_out.send(OutgoingMessage {
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
        self.plat.draw_image(&Point::init(0, 0), &self.background.bounds(), &self.background);
        let MAGENTA:ARGBColor = ARGBColor::new_rgb(255, 0, 255);
        self.draw_windows();

        //draw the fps amount
        let mut total = 0;
        for dur in &self.fps {
            total += dur;
        }
        let avg_frame_length = (total as f64)/(self.fps.len() as f64);
        self.debug_buffer.clear(&BLACK);
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
        let MAGENTA:ARGBColor = ARGBColor::new_rgb(255, 0, 255);
        for win_id in &self.state.window_order {
            if let Some(win) = self.state.lookup_window(*win_id) {
                let (wc, tc) = if self.state.is_focused_window(win) {
                    (FOCUSED_WINDOW_COLOR, FOCUSED_TITLEBAR_COLOR)
                } else {
                    (WINDOW_COLOR, TITLEBAR_COLOR)
                };
                // draw the titlebar
                self.plat.fill_rect(win.titlebar_bounds(), &tc);
                self.plat.fill_rect(win.close_button_bounds(), &WINDOW_BUTTON_COLOR);
                //draw the content
                let bd = win.content_bounds();
                self.plat.fill_rect(bd, &MAGENTA);
                self.plat.draw_image(&win.content_bounds().position(), &win.backbuffer.bounds(), &win.backbuffer);
                // draw the resize button
                self.plat.fill_rect(win.resize_bounds(), &MAGENTA);

            }
        }
    }
}
