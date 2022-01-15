use std::error::Error;
use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use sdl2::{EventPump, Sdl};
use sdl2::pixels::Color;
use sdl2::rect::Rect as SDLRect;
use common::{ARGBColor, Rect as CommonRect, Rect};

pub struct Plat {
    pub sdl_context: Sdl,
    pub canvas: WindowCanvas,
    pub event_pump: EventPump,
}

impl Plat {
    pub fn init() -> Result<Plat, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        println!("verison is {}", sdl2::version::version());
        println!("current driver is {:}",video_subsystem.current_video_driver());
        let display_count = video_subsystem.num_video_displays()?;
        println!("display count {:}",display_count);
        for d in sdl2::video::drivers() {
            println!("video driver {}",d);
        }

        for d in sdl2::render::drivers() {
            println!("render driver {:?}",d);
        }
        let window = video_subsystem
            .window("rust-sdl2 demo: Video", 512*2, 320*2)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas_builder = window.into_canvas();
        let mut canvas = canvas_builder.build().map_err(|e| e.to_string())?;
        let mut event_pump = sdl_context.event_pump()?;
        Ok(Plat {
            sdl_context:sdl_context,
            canvas:canvas,
            event_pump:event_pump,
        })
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

    pub fn service_loop(&mut self) {
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
                _ => {}
            }
        }
        self.canvas.set_draw_color(Color::RED);
        self.canvas.fill_rect(SDLRect::new(0,0,100,100));
        self.canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    pub fn fill_rect(&mut self, rect:CommonRect, color:&ARGBColor) {
        self.canvas.set_draw_color(Color::RED);
        self.canvas.fill_rect(SDLRect::new(rect.x, rect.y, rect.w as u32, rect.h as u32));
    }

    pub fn shutdown(&mut self) {

    }
}

