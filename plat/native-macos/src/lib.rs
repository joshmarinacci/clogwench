use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool};
use log::{error};
use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};
use sdl2::event::Event;
use sdl2::render::{Texture, TextureAccess, TextureCreator, WindowCanvas};
use sdl2::video::{WindowContext};
use sdl2::{EventPump};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect as SDLRect;
use uuid::Uuid;
use common::events::{KeyDownEvent, ModifierState, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use common::{APICommand, IncomingMessage};
use gfx::graphics::{ARGBColor, GFXBuffer, PixelLayout, Point, Rect};
use gfx::graphics::Rect as CommonRect;

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
    let mut canvas:WindowCanvas = window.into_canvas().software().build().map_err(|e| e.to_string())?;
    println!("using scale {}x{} scale={}",w,h, (scale as f32)*1.0);
    canvas.set_scale((scale as f32)*1.0,(scale as f32)*1.0)?;

    Ok(Plat {
        stop,
        textures: Default::default(),
        creator: canvas.texture_creator(),
        canvas,
        event_pump:sdl_context.event_pump()?,
        sender,
    })
}

impl Plat {
    pub fn get_preferred_pixel_layout(&self) -> &PixelLayout {
        &PixelLayout::ARGB()
    }

    pub fn get_screen_bounds(&self) -> CommonRect {
        let r2 = self.canvas.viewport();
        CommonRect {
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
                            trace: false,
                            timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                            source: Default::default(),
                            command: APICommand::KeyDown(KeyDownEvent{
                                app_id: Default::default(),
                                window_id: Default::default(),
                                key: sdl_util::sdl_to_common(kk, keymod),
                                mods: ModifierState::empty(),
                            })
                        };
                        if let Err(e) = self.sender.send(cmd) {
                            error!("error sending {}",e);
                        }
                    }
                },
                Event::MouseButtonDown { x, y,mouse_btn, .. } => {
                    let (x, y) = scale_mouse_to_canvas(&self.canvas,x,y);
                    let cmd = IncomingMessage {
                        trace: false,
                        timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
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
                    let (x, y) = scale_mouse_to_canvas(&self.canvas,x,y);
                    let cmd = IncomingMessage {
                        trace: false,
                        timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
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
                    let (x, y) = scale_mouse_to_canvas(&self.canvas,x,y);
                    let cmd = IncomingMessage {
                        trace: false,
                        timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                        source: Default::default(),
                        command: APICommand::MouseMove(MouseMoveEvent {
                            app_id: Default::default(),
                            window_id: Default::default(),
                            original_timestamp: 0,
                            button: MouseButton::Primary,
                            x,
                            y,
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
            let dst: SDLRect = SDLRect::new(dst_pos.x, dst_pos.y, src_bounds.w as u32, src_bounds.h as u32);
            sync_texture(&mut self.canvas, tex, src_buf);
            let src: SDLRect = SDLRect::new(src_bounds.x, src_bounds.y, src_bounds.w as u32, src_bounds.h as u32);
            self.canvas.copy(tex, src, dst);
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

fn scale_mouse_to_canvas(canvas:&WindowCanvas, x: i32, y: i32) -> (i32,i32) {
    let (scx, scy) = canvas.scale();
    let x = ((x as f32)/scx) as i32;
    let y = ((y as f32)/scy) as i32;
    (x,y)
}

fn sync_texture(can: &mut WindowCanvas, tx: &mut Texture, img: &GFXBuffer) {
    let rect = sdl2::rect::Rect::new(0,0,img.width,img.height);
    let pitch:usize = (img.width * 4) as usize;
    tx.update(rect, &img.data, pitch).unwrap();
}

