/*
the new minifb based plat


 */
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use minifb::{Key, MouseButton, MouseMode, Scale, Window, WindowOptions};
use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;
use minifb::Key::LeftShift;
use common::events::{KeyDownEvent, KeyUpEvent, ModifierState, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use common::generated::KeyCode;
use common::{APICommand, IncomingMessage};
use gfx::graphics::{ARGBColor, GFXBuffer, PixelLayout, Point, Rect};

// const WIDTH: usize = 640;
// const HEIGHT: usize = 360;
type KeyVec = Rc<RefCell<Vec<(Key,bool)>>>;

pub struct Plat {
    sender:Sender<IncomingMessage>,
    screen_size:Rect,
    layout:PixelLayout,
    pub window: Window,
    mouse_down:bool,
    pub buffer: Vec<u32>,
    pub keys_data2: KeyVec,
    pub mod_state: ModifierState,
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
        let buffer_bounds = Rect::from_ints(0, 0, width as i32, height as i32);
        let fill_bounds = buffer_bounds.intersect(rect);
        let mut row = vec![0; fill_bounds.w as usize];
        row.fill(color);
        for (j, row_slice) in self.buffer.chunks_exact_mut(width).enumerate() {
            let j = j as i32;
            if j < fill_bounds.y {
                continue;
            }
            if j >= fill_bounds.y + fill_bounds.h {
                continue;
            }
            let (_, after) = row_slice.split_at_mut((fill_bounds.x) as usize);
            let (middle, _) = after.split_at_mut((fill_bounds.w) as usize);
            middle.copy_from_slice(&row);
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
                        trace: true,
                        timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
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
                        trace: false,
                        timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
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
                    trace: false,
                    timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
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

        for (key, down) in self.keys_data2.borrow_mut().iter() {
            println!("Code point: {:?} state {}", key, down);
            if is_modifier_key(key) {
                match key {
                    LeftShift => self.mod_state.shift = *down,
                    // RightShift => self.mod_state.shift = *down,
                    _ => {}
                }
                println!("state {:?}",self.mod_state);
                continue;
            } else {
                let keycode = minifb_to_KeyCode(key);
                let command:APICommand = if *down {
                    APICommand::KeyDown(KeyDownEvent {
                        app_id: Default::default(),
                        window_id: Default::default(),
                        key: keycode,
                        mods: self.mod_state.clone(),
                    })
                } else {
                    APICommand::KeyUp(KeyUpEvent {
                        app_id: Default::default(),
                        window_id: Default::default(),
                        key: keycode,
                        mods: self.mod_state.clone(),
                    })
                };
                let cmd = IncomingMessage {
                    trace:false,
                    timestamp_usec: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(),
                    source: Default::default(),
                    command,
                };
                if let Err(e) = self.sender.send(cmd) {
                    println!("error sending key down out {:?}",e);
                }
            }
        }
        self.keys_data2.borrow_mut().clear();
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
    fn update_modifier_state(&mut self, p0: &Key, p1: &bool) {
        self.mod_state.shift = true
    }
}

fn is_modifier_key(key: &Key) -> bool {
    match key {
        Key::LeftShift => true,
        Key::RightShift => true,
        Key::LeftCtrl => true,
        Key::RightCtrl => true,
        Key::LeftAlt => true,
        Key::RightAlt => true,
        Key::LeftSuper => true,
        Key::RightSuper => true,
        _ => false
    }
}

fn minifb_to_KeyCode(id: &Key) -> KeyCode {
    match id {
        Key::A => KeyCode::LETTER_A,
        Key::B => KeyCode::LETTER_B,
        Key::C => KeyCode::LETTER_C,
        Key::D => KeyCode::LETTER_D,
        Key::E => KeyCode::LETTER_E,
        Key::F => KeyCode::LETTER_F,
        Key::G => KeyCode::LETTER_G,

        Key::H => KeyCode::LETTER_H,
        Key::I => KeyCode::LETTER_I,
        Key::J => KeyCode::LETTER_J,
        Key::K => KeyCode::LETTER_K,
        Key::L => KeyCode::LETTER_L,
        Key::M => KeyCode::LETTER_M,
        Key::N => KeyCode::LETTER_N,
        Key::O => KeyCode::LETTER_O,
        Key::P => KeyCode::LETTER_P,

        Key::Q => KeyCode::LETTER_Q,
        Key::R => KeyCode::LETTER_R,
        Key::S => KeyCode::LETTER_S,
        Key::T => KeyCode::LETTER_T,
        Key::U => KeyCode::LETTER_U,
        Key::V => KeyCode::LETTER_V,
        Key::W => KeyCode::LETTER_W,
        Key::X => KeyCode::LETTER_X,
        Key::Y => KeyCode::LETTER_Y,
        Key::Z => KeyCode::LETTER_Z,

        Key::Key0 => KeyCode::DIGIT_0,
        Key::Key1 => KeyCode::DIGIT_1,
        Key::Key2 => KeyCode::DIGIT_2,
        Key::Key3 => KeyCode::DIGIT_3,
        Key::Key4 => KeyCode::DIGIT_4,
        Key::Key5 => KeyCode::DIGIT_5,
        Key::Key6 => KeyCode::DIGIT_6,
        Key::Key7 => KeyCode::DIGIT_7,
        Key::Key8 => KeyCode::DIGIT_8,
        Key::Key9 => KeyCode::DIGIT_9,

        Key::Right => KeyCode::ARROW_RIGHT,
        Key::Left  => KeyCode::ARROW_LEFT,
        Key::Down  => KeyCode::ARROW_DOWN,
        Key::Up    => KeyCode::ARROW_UP,

        Key::Delete => KeyCode::DELETE,
        Key::Backspace => KeyCode::BACKSPACE,

        Key::Apostrophe => KeyCode::QUOTE,
        Key::Backquote => KeyCode::BACKQUOTE,
        Key::Backslash => KeyCode::BACKSLASH,
        Key::Comma => KeyCode::COMMA,
        Key::Equal => KeyCode::EQUALS,

        _ => KeyCode::UNKNOWN,
    }
}

struct Input {
    keys: KeyVec,
}

impl Input {
    fn new(data: &KeyVec) -> Input {
        Input {
            keys: data.clone(),
        }
    }
}
impl minifb::InputCallback for Input {
    fn add_char(&mut self, uni_char: u32) {
        // self.keys.borrow_mut().push(uni_char);
    }
    fn set_key_state(&mut self, key: Key, state: bool) {
        // println!("key {:?} state={:?}", key, state);
        self.keys.borrow_mut().push((key,state));
    }
}
pub fn make_plat<'a>(stop:Arc<AtomicBool>, sender: Sender<IncomingMessage>, width:u32, height:u32, scale:u32) -> Result<Plat, String> {
    // println!("making minibuf plat scale settings");
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
    // let keys_data = KeyVec::new(RefCell::new(Vec::new()));
    let keys_data2 = KeyVec::new(RefCell::new(Vec::new()));
    let input = Box::new(Input::new(&keys_data2));
    window.set_input_callback(input);
    window.limit_update_rate(Some(std::time::Duration::from_millis(16))); //60fps

    return Ok(Plat {
        sender,
        buffer: vec![0; screen_size.w as usize * screen_size.h as usize],
        window:window,
        screen_size: screen_size,
        layout:PixelLayout::ARGB(),
        mouse_down:false,
        keys_data2,
        mod_state:ModifierState::empty(),
    });
}
