extern crate sdl2;
use zmq;
use std::{str};
use std::time::{Duration, Instant};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::MouseButton;
use gfx::graphics::{ARGBColor, GFXBuffer, PixelLayout, Point, Rect, Size};
use sdl2::rect::Rect as SDLRect;
use sdl2::render::TextureAccess;
use common::events::ModifierState;
use sdl_util::sdl_to_common;

fn main() {
    // println!("starting window side");

    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::DEALER).unwrap();
    socket.bind("tcp://127.0.0.1:3000").unwrap();
    println!("window bound to endpoint");
    let mut msg = zmq::Message::new();


    // let window = sdl.video.createWindow({ resizable: false, width:800, height:600 })
    // let canvas = Canvas.createCanvas(window.pixelWidth,window.pixelHeight)
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 300, 300)
        .position_centered()
        .resizable()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut texture_bounds: SDLRect = SDLRect::new(0, 0, 800, 600);

    let tex_creator = canvas.texture_creator();
    let mut tex = tex_creator.create_texture(
        PixelFormatEnum::ABGR8888,
        TextureAccess::Target, texture_bounds.w as u32, texture_bounds.h as u32).unwrap();

    canvas.copy(&tex, None, texture_bounds).unwrap();

    
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        // println!("look for input events");
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'running;
                }
                Event::KeyDown { keycode: Some(code), keymod,.. } => {
                    let kc = sdl_to_common(code,keymod);
                    let mods:ModifierState = create_mod(keymod);
                    socket.send_multipart(&["key-down",
                        serde_json::to_string(&kc).unwrap().as_str(),
                        serde_json::to_string(&mods).unwrap().as_str()], 0).unwrap()
                },
                Event::MouseButtonDown { x, y , mouse_btn, ..} => {
                    let point = Point::init(x,y);
                    let mut mouse_button = "Primary";
                    if(mouse_btn == MouseButton::Right) {  mouse_button = "Secondary";  }
                    socket.send_multipart(&["mouse-down", 
                        serde_json::to_string(&point).unwrap().as_str(), 
                        mouse_button], 0).unwrap()
                }
                Event::MouseButtonUp { x,y,mouse_btn, ..} => {
                    let point = Point::init(x,y);
                    let mut mouse_button = "Primary";
                    if(mouse_btn == MouseButton::Right) {  mouse_button = "Secondary";  }
                    socket.send_multipart(&["mouse-up",
                        serde_json::to_string(&point).unwrap().as_str(),
                        mouse_button], 0).unwrap()
                }
                Event::MouseMotion {x,y,mousestate,..} => {
                    let point = Point::init(x,y);
                    socket.send_multipart(&["mouse-move",
                        serde_json::to_string(&point).unwrap().as_str(),
                        ], 0).unwrap()
                }
                Event::Window { timestamp, window_id, win_event } => {
                    match win_event {
                        WindowEvent::Resized(w,h) => {
                            let size = Size::init(w,h);
                            socket.send_multipart(&["window-resized",
                                serde_json::to_string(&size).unwrap().as_str(),
                                ],0).unwrap()
                        },
                        WindowEvent::Close => {
                            socket.send_multipart(&["window-closed"],0).unwrap()
                        }
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        // println!("look for messages");
        if socket.poll(zmq::POLLIN, 10).expect("client failed polling") > 0 {
            // println!("receiving data");
            // let msg = socket.recv_multipart(0).expect("failed");
            socket.recv(&mut msg,0).unwrap();
            // println!("got {}", msg.as_str().unwrap());
            if(msg.as_str().unwrap().eq("open-window")) {
                socket.recv(&mut msg,0).unwrap();
                // println!("open window size is {}", msg.as_str().unwrap());
                if let Ok(size) = serde_json::from_str::<Size>(msg.as_str().unwrap()) {
                    // println!("the size is {}",size);
                    let rect = sdl2::rect::Rect::new(0, 0, size.w as u32, size.h as u32);
                    canvas.set_viewport(rect)
                }
            }
            if(msg.as_str().unwrap().eq("repaint")) {
                // println!("Received a repaint message");
                socket.recv(&mut msg,0).unwrap();
                // println!("size should be {}", msg.as_str().unwrap());
                if let Ok(size) = serde_json::from_str::<Size>(msg.as_str().unwrap()) {
                    let rect = sdl2::rect::Rect::new(0, 0, size.w as u32, size.h as u32);
                    // canvas.set_viewport(rect)
                    // println!("the repaint size is {:?}",rect);
                    // println!("current texture size is {:?}", texture_bounds );
                    if(texture_bounds.w != rect.w ) {
                        // println!("different size, must recreate");
                        texture_bounds.w = rect.w;
                        texture_bounds.h = rect.h;
                        tex = tex_creator.create_texture(
                            PixelFormatEnum::ABGR8888,
                            TextureAccess::Target, texture_bounds.w as u32, texture_bounds.h as u32).unwrap();
                    }
                    socket.recv(&mut msg,0).unwrap();
                    let arr = msg.to_vec();
                    let pitch:usize = (texture_bounds.w * 4) as usize;
                    tex.update(texture_bounds, &arr, pitch).unwrap();
                }
            }
        }
        canvas.copy(&tex, None, texture_bounds).unwrap();
        // canvas.set_draw_color(Color::RED);
        // canvas.fill_rect(SDLRect::new(0,0,100,100));
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn create_mod(keymod: Mod) -> ModifierState {
    let mut mods:ModifierState = ModifierState {
        shift: false,
        ctrl: false,
        alt: false,
        meta: false,
    };
    if (keymod == Mod::LSHIFTMOD || keymod == Mod::RSHIFTMOD) {
        mods.shift = true
    }
    if (keymod == Mod::LCTRLMOD || keymod == Mod::RCTRLMOD) {
        mods.ctrl = true
    }
    if (keymod == Mod::LALTMOD || keymod == Mod::RALTMOD) {
        mods.alt = true
    }
    if (keymod == Mod::LGUIMOD || keymod == Mod::RGUIMOD) {
        mods.meta = true
    }
    mods
}
