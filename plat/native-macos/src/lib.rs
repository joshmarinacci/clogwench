use std::collections::HashMap;
use log::{error, info};
use std::sync::mpsc::Sender;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Texture, WindowCanvas, TextureCreator, TextureAccess};
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, Sdl};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect as SDLRect;

use uuid::Uuid;
use common::{APICommand, ARGBColor, IncomingMessage, Rect as CommonRect, Rect};
use common::events::{MouseButton, MouseMoveEvent};
use common::graphics::GFXBuffer;


pub struct TM {
    pub event_pump: EventPump,
    pub canvas: WindowCanvas,
    pub textures: HashMap<Uuid, Texture>,
    pub creator: TextureCreator<WindowContext>,
    pub sender: Sender<IncomingMessage>,
}

pub fn make_plat<'a>(sender: Sender<IncomingMessage>) -> Result<TM, String> {
    let sdl_context = sdl2::init().unwrap();
    let window = sdl_context.video()?
        .window("rust-sdl2 demo: Video", 512 * 2, 320 * 2)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let canvas:WindowCanvas = window.into_canvas().software().build().map_err(|e| e.to_string())?;

    return Ok(TM {
        textures: Default::default(),
        creator: canvas.texture_creator(),
        canvas: canvas,
        event_pump:sdl_context.event_pump()?,
        sender: sender,
    });
}

impl TM {

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
        info!("mac doing events");
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    println!("quitting");
                    break;
                },
                // Event::KeyDown {keycode,keymod,..} => self.process_keydown(keycode, keymod, windows,output),
                // Event::MouseButtonDown { x, y,mouse_btn, .. } => self.process_mousedown(x,y,mouse_btn, windows, output),
                // Event::MouseButtonUp {x,y,mouse_btn,..} =>  self.process_mouseup(x,y,mouse_btn,windows,output),
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
                    self.sender.send(cmd).unwrap();
                }
                _ => {}
            }
        }
        info!("mac done events");
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
        }
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
    }

    pub fn shutdown(&mut self) {

    }

    pub fn register_image2(&mut self) {
        let tc = self.canvas.texture_creator();
        let tex = tc.create_texture(
            PixelFormatEnum::RGBA8888,
            TextureAccess::Target, 20, 20).unwrap();
        self.textures.insert(Uuid::new_v4(), tex);
    }
}

