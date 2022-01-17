use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use log::{error, info};
use std::sync::mpsc::Sender;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, WindowCanvas, TextureCreator, TextureAccess};
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, Sdl};
use sdl2::keyboard::Keycode::D;
use sdl2::pixels::{Color, PixelFormat, PixelFormatEnum};
use sdl2::rect::Rect as SDLRect;

use uuid::Uuid;
use common::{APICommand, ARGBColor, IncomingMessage, Rect as CommonRect, Rect};
use common::events::{MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use common::graphics::{ColorDepth, GFXBuffer};


pub struct Plat {
    pub event_pump: EventPump,
    pub canvas: WindowCanvas,
    pub textures: HashMap<Uuid, Texture>,
    pub creator: TextureCreator<WindowContext>,
    pub sender: Sender<IncomingMessage>,
    pub stop: Arc<AtomicBool>,
}

pub fn make_plat<'a>(stop:Arc<AtomicBool>, sender: Sender<IncomingMessage>) -> Result<Plat, String> {
    let sdl_context = sdl2::init().unwrap();
    let window = sdl_context.video()?
        .window("rust-sdl2 demo: Video", 512 * 2, 320 * 2)
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
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    info!("quitting");
                    self.stop.store(true, Ordering::Relaxed);
                    break;
                },
                // Event::KeyDown {keycode,keymod,..} => self.process_keydown(keycode, keymod, windows,output),
                Event::MouseButtonDown { x, y,mouse_btn, .. } => {
                    let cmd = IncomingMessage {
                        source: Default::default(),
                        command: APICommand::MouseDown(MouseDownEvent{
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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    pub fn fill_rect(&mut self, rect: CommonRect, color: &ARGBColor) {
        let c2 = Color::RGB(color.r, color.g, color.b);
        self.canvas.set_draw_color(c2);
        self.canvas.fill_rect(SDLRect::new(rect.x, rect.y, rect.w as u32, rect.h as u32));
    }

    pub fn draw_rect(&mut self, rect: Rect, color: &ARGBColor, width: i32) {
        let c2 = Color::RGB(color.r, color.g, color.b);
        self.canvas.set_draw_color(c2);
        self.canvas.fill_rect(SDLRect::new(rect.x, rect.y, rect.w as u32, rect.h as u32));
    }
    pub fn draw_image(&mut self, x: i32, y: i32, img: &GFXBuffer) {
        if let Some(tex) = self.textures.get(&img.id) {
            let dst: SDLRect = SDLRect::new(x, y, img.width, img.height);
            self.canvas.copy(tex, None, dst);
        } else {
            error!("no image found for {}",img.id);
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
            PixelFormatEnum::ARGB8888,
            TextureAccess::Target, img.width, img.height).unwrap();
        self.textures.insert(img.id, tex);

        if let Some(tx) = self.textures.get_mut(&img.id) {
            self.canvas.with_texture_canvas(tx, |can| {
                for i in 0..img.width {
                    for j in 0..img.height {
                        let n:usize = ((j * img.width + i) * 4) as usize;
                        match img.bitdepth {
                            ColorDepth::CD16() => {}
                            ColorDepth::CD24() => {}
                            ColorDepth::CD32() => {
                                //let px = img.get_pixel_32argb(i,j);
                                let ve = img.get_pixel_vec_argb(i as u32,j as u32);
                                let col = Color::RGBA(ve[1],ve[2],ve[3], ve[0]);
                                can.set_draw_color(col);
                                can.fill_rect(SDLRect::new(i as i32, j as i32, 1, 1));
                            }
                        }
                    }
                }
            });
        }
    }
}

