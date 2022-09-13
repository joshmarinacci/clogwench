use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use log::{error, info};
use std::sync::mpsc::Sender;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, TextureAccess, TextureCreator, WindowCanvas};
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, Sdl};
use sdl2::keyboard::Keycode::D;
use sdl2::pixels::{Color, PixelFormat, PixelFormatEnum};
use sdl2::rect::Rect as SDLRect;

use uuid::Uuid;
use common::{APICommand, ARGBColor, IncomingMessage, Point, Rect as CommonRect, Rect};
use common::events::{KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use common::generated::KeyCode;
use common::graphics::{GFXBuffer, PixelLayout};

mod sdl_to_common;


pub struct Plat {
    pub event_pump: EventPump,
    pub canvas: WindowCanvas,
    pub textures: HashMap<Uuid, Texture>,
    pub creator: TextureCreator<WindowContext>,
    pub sender: Sender<IncomingMessage>,
    pub stop: Arc<AtomicBool>,
}

pub fn make_plat<'a>(stop:Arc<AtomicBool>, sender: Sender<IncomingMessage>, w:u32, h:u32, scale:u32) -> Result<Plat, String> {
    let sdl_context = sdl2::init().unwrap();
    let window = sdl_context.video()?
        .window("rust-sdl2 demo: Video", w * scale, h* scale)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let canvas:WindowCanvas = window.into_canvas().software().build().map_err(|e| e.to_string())?;

    return Ok(Plat {
        stop:stop,
        textures: Default::default(),
        creator: canvas.texture_creator(),
        canvas: canvas,
        event_pump:sdl_context.event_pump()?,
        sender: sender,
    });
}

impl Plat {
    pub fn get_preferred_pixel_layout(&self) -> &PixelLayout {
        &PixelLayout::ARGB()
    }

    pub fn get_screen_bounds(&self) -> CommonRect {
        let r2 = self.canvas.viewport();
        return CommonRect {
            x: r2.x(),
            y: r2.y(),
            w: r2.width() as i32,
            h: r2.height() as i32,
        }
    }

    pub fn service_input(&mut self) {
        // info!("mac doing events");
        for event in self.event_pump.poll_iter() {
            match event {
                // Event::Quit { .. }
                // | Event::KeyDown {
                //     keycode: Some(Keycode::Escape),
                //     ..
                // } => {
                //     info!("quitting");
                //     self.stop.store(true, Ordering::Relaxed);
                //     break;
                // },
                Event::KeyDown {keycode,keymod,scancode,..} => {
                    if let Some(kk) = keycode {
                        // println!("keycode is {}",kk);
                        // println!("scancode is {:?}",scancode);
                        // println!("mod is {}",keymod);
                        let cmd = IncomingMessage {
                            source: Default::default(),
                            command: APICommand::KeyDown(KeyDownEvent{
                                app_id: Default::default(),
                                window_id: Default::default(),
                                original_timestamp: 0,
                                code: sdl_to_common::sdl_to_common(kk,keymod),
                                key: sdl_to_common::sdl_to_common_letter(kk,keymod),
                            })
                        };
                        if let Err(e) = self.sender.send(cmd) {
                            error!("error sending {}",e);
                        }
                    }
                },
                Event::MouseButtonDown { x, y,mouse_btn, .. } => {
                    let cmd = IncomingMessage {
                        source: Default::default(),
                        command: APICommand::MouseDown(MouseDownEvent{
                            app_id: Default::default(),
                            window_id: Default::default(),
                            original_timestamp: 0,
                            button: MouseButton::Primary,
                            x,
                            y
                        })
                    };
                    if let Err(e) = self.sender.send(cmd) {
                        error!("error sending {}",e);
                    }
                },
                Event::MouseButtonUp {x,y,mouse_btn,..} =>  {
                    let cmd = IncomingMessage {
                        source: Default::default(),
                        command: APICommand::MouseUp(MouseUpEvent{
                            app_id: Default::default(),
                            window_id: Default::default(),
                            original_timestamp: 0,
                            button: MouseButton::Primary,
                            x,
                            y
                        })
                    };
                    if let Err(e) = self.sender.send(cmd) {
                        error!("error sending {}",e);
                    }
                },
                Event::MouseMotion {
                    timestamp, window_id, which, mousestate, x, y, xrel, yrel
                } => {
                    let cmd = IncomingMessage {
                        source: Default::default(),
                        command: APICommand::MouseMove(MouseMoveEvent {
                            app_id: Default::default(),
                            window_id: Default::default(),
                            original_timestamp: 0,
                            button: MouseButton::Primary,
                            x: x as i32,
                            y: y as i32
                        })
                    };
                    // info!("about to send out {:?}",cmd);
                    if let Err(e) = self.sender.send(cmd) {
                        error!("error sending mouse motion out {:?}",e);
                    }
                }
                _ => {}
            }
        }
        // info!("mac done events");
    }

    pub fn service_loop(&mut self) {
        // self.canvas.set_draw_color(Color::RED);
        // self.canvas.fill_rect(SDLRect::new(0,0,100,100));
        self.canvas.present();
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    pub fn fill_rect(&mut self, rect: CommonRect, color: &ARGBColor) {
        let c2 = Color::RGB(color.r, color.g, color.b);
        self.canvas.set_draw_color(c2);
        self.canvas.fill_rect(SDLRect::new(rect.x, rect.y, rect.w as u32, rect.h as u32));
    }

    pub fn draw_image(&mut self, dst_pos:&Point, src_bounds: &Rect, src_buf: &GFXBuffer) {
        if let Some(tex) = self.textures.get_mut(&src_buf.id) {
            let dst: SDLRect = SDLRect::new(dst_pos.x, dst_pos.y, src_buf.width, src_buf.height);
            sync_texture(&mut self.canvas, tex, src_buf);
            self.canvas.copy(tex, None, dst);
        } else {
            error!("no image found for {}",src_buf.id);
        }
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
    }

    pub fn shutdown(&mut self) {

    }

    pub fn register_image2(&mut self, img:&GFXBuffer) {
        let tex_creator = self.canvas.texture_creator();
        let tex = tex_creator.create_texture(
            PixelFormatEnum::RGBA8888,
            TextureAccess::Target, img.width, img.height).unwrap();
        self.textures.insert(img.id, tex);

        if let Some(tx) = self.textures.get_mut(&img.id) {
            sync_texture(&mut self.canvas, tx, img);
        }
    }
    pub fn unregister_image2(&mut self, img:&GFXBuffer) {
        self.textures.remove(&img.id);
    }
}

fn sync_texture(can: &mut WindowCanvas, tx: &mut Texture, img: &GFXBuffer) {
    let rect = sdl2::rect::Rect::new(0,0,img.width,img.height);
    let pitch:usize = (img.width * 4) as usize;
    tx.update(rect, &img.data, pitch).unwrap();
}

