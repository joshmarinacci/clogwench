/*
the new minifb based plat


 */
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use minifb::{Key, MouseButton, MouseMode, Scale, Window, WindowOptions};
use common::{ARGBColor, IncomingMessage, Rect, BLACK, Point, APICommand};
use std::sync::mpsc::Sender;
use log::info;
use common::events::{MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use common::graphics::{GFXBuffer, PixelLayout};

// const WIDTH: usize = 640;
// const HEIGHT: usize = 360;

pub struct Plat {
    sender:Sender<IncomingMessage>,
    screen_size:Rect,
    layout:PixelLayout,
    pub window: Window,
    mouse_down:bool,
    pub buffer: Vec<u32>,
}

impl Plat {
    pub fn clear(&mut self) {
        for i in self.buffer.iter_mut() {
            *i = 0xFF000000;
        }
    }
    pub fn fill_rect(&mut self, rect: Rect, fill_color: &ARGBColor) {
        let (width, height) = self.window.get_size();
        let color = fill_color.to_argb_u32();
        // println!("fill rect {}x{} rect = {} len = {} vs {} color={:#x}", width, height, rect,
        //          width*height, self.buffer.len(),
        //     color,
        // );
        let ry = rect.y as usize;
        let rx = rect.x as usize;
        let rh = rect.h as usize;
        let rw = rect.w as usize;
        for y in 0..rh {
            for x in 0..rw {
                self.buffer[((ry + y) * width) + rx + x] = color
            }
        }
    }
    pub fn draw_image(&mut self, dst_pos: &Point, src_bounds: &Rect, src_buf: &GFXBuffer) {
        let (width, height) = self.window.get_size();
        // println!("src format {:?}", src_buf.layout);
        for j in src_bounds.y .. src_bounds.y + src_bounds.h {
            for i in src_bounds.x .. src_bounds.x + src_bounds.w {
                let v = src_buf.get_pixel_u32_argb(
                    ( (i - src_bounds.x) as u32 % src_buf.width) as i32,
                    ( (j - src_bounds.y) as u32 % src_buf.height) as i32);
                let dx = (i + dst_pos.x) as usize;
                let dy = (j + dst_pos.y) as usize;
                if dx >= 0 && dx < width && dy >= 0 && dy < height {
                    self.buffer[dy * width + dx] = v
                }
            }
        }
    }
    pub fn unregister_image2(&self, p0: &GFXBuffer) {
    }
    pub fn service_loop(&mut self) {
        if self.window.is_open() {
            self.window
                .update_with_buffer(&self.buffer,
                                    (self.screen_size.w as usize) , (self.screen_size.h as usize) )
                .unwrap();
        } else {
            println!("we need to turn off the window");
        }
    }
    pub fn service_input(&mut self) {
        if let Some((x, y)) = self.window.get_mouse_pos(MouseMode::Discard) {
            let x = x.floor() as i32;
            let y = y.floor() as i32;
            // println!("mouse pos is {}x{}",x,y);
            let current_mouse_down = self.window.get_mouse_down(MouseButton::Left);
            if current_mouse_down != self.mouse_down {
                if current_mouse_down {
                    self.mouse_down = current_mouse_down;
                    let cmd = IncomingMessage {
                        source: Default::default(),
                        command: APICommand::MouseDown(MouseDownEvent {
                            app_id: Default::default(),
                            window_id: Default::default(),
                            original_timestamp: 0,
                            button: common::events::MouseButton::Primary,
                            x,
                            y,
                        })
                    };
                    // info!("about to send out {:?}",cmd);
                    if let Err(e) = self.sender.send(cmd) {
                        println!("error sending mouse down out {:?}",e);
                    }
                } else {
                    self.mouse_down = current_mouse_down;
                    let cmd = IncomingMessage {
                        source: Default::default(),
                        command: APICommand::MouseUp(MouseUpEvent {
                            app_id: Default::default(),
                            window_id: Default::default(),
                            original_timestamp: 0,
                            button: common::events::MouseButton::Primary,
                            x,
                            y,
                        })
                    };
                    // info!("about to send out {:?}",cmd);
                    if let Err(e) = self.sender.send(cmd) {
                        println!("error sending mouse up out {:?}",e);
                    }
                }
            } else {
                let cmd = IncomingMessage {
                    source: Default::default(),
                    command: APICommand::MouseMove(MouseMoveEvent {
                        app_id: Default::default(),
                        window_id: Default::default(),
                        original_timestamp: 0,
                        button: common::events::MouseButton::Primary,
                        x,
                        y,
                    })
                };
                // info!("about to send out {:?}",cmd);
                if let Err(e) = self.sender.send(cmd) {
                    println!("error sending mouse motion out {:?}", e);
                }
            }
        }
    }
    pub fn get_preferred_pixel_layout(&self) -> &PixelLayout {
        return &self.layout
    }
    pub fn shutdown(&self) {
        println!("stopping");
    }
    pub fn register_image2(&self, img: &GFXBuffer) {
    }
    pub fn get_screen_bounds(&self) -> Rect {
        self.screen_size
    }
}

pub fn make_plat<'a>(stop:Arc<AtomicBool>, sender: Sender<IncomingMessage>, width:u32, height:u32, scale:u32) -> Result<Plat, String> {
    println!("making minibuf plat scale settings");
    let screen_size = Rect::from_ints(0,0,640,480);

    let mut window = match Window::new(
        "cool app",
        screen_size.w as usize,
        screen_size.h as usize,
        WindowOptions {
            // resize: false,
            // scale: Scale::X2,
            ..WindowOptions::default()
        },
    ) {
        Ok(win) => win,
        Err(err) => {
            println!("Unable to create window {}", err);
            panic!("unable to create window");
        }
    };

    return Ok(Plat {
        sender,
        buffer: vec![0; screen_size.w as usize * screen_size.h as usize],
        window:window,
        screen_size: screen_size,
        layout:PixelLayout::ARGB(),
        mouse_down:false,
    });
}
