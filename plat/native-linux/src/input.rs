use std::cmp::max;
use evdev::{AbsoluteAxisType, Device, EventType, InputEventKind, Key, RelativeAxisType};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use common::{APICommand, IncomingMessage, Rect};
use common::events::*;

use std::thread;
use log::{info, warn};
use common::events::{KeyCode, MouseButton, MouseMoveEvent};

pub fn find_keyboard() -> Option<evdev::Device> {
    let mut devices = evdev::enumerate().collect::<Vec<_>>();
    devices.reverse();
    for (i, d) in devices.iter().enumerate() {
        if d.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
            println!("found keyboard device {}",d.name().unwrap_or("Unnamed device"));
            return devices.into_iter().nth(i);
        }
    }
    None
}

pub fn find_mouse() -> Option<evdev::Device> {
    let mut devices = evdev::enumerate().collect::<Vec<_>>();
    devices.reverse();
    for (i, d) in devices.iter().enumerate() {
        for typ in d.supported_events().iter() {
            println!("   type {:?}",typ);
        }
        if d.supported_events().contains(EventType::RELATIVE) {
            println!("found a device with relative tools-input {}", d.name().unwrap_or("unnamed device"));
            return devices.into_iter().nth(i);
        }
        if d.supported_events().contains(EventType::ABSOLUTE) {
            println!("found a device with absolute tools-input: {}", d.name().unwrap_or("Unnamed device"));
            return devices.into_iter().nth(i);
        }
        // if d.supported_relative_axes().map_or(false, |axes| axes.contains(RelativeAxisType::REL_X)) {
        //     println!("found a device with relative tools-input: {}", d.name().unwrap_or("Unnamed device"));
        //     return devices.into_iter().nth(i);
        // }
    }
    None
}

fn linuxkernel_to_KeyCode(code:u16) -> KeyCode {
    let key = Key::new(code);
    match key {
        Key::KEY_RESERVED => KeyCode::RESERVED,
        Key::KEY_ESC => KeyCode::ESC,
        Key::KEY_LEFT => KeyCode::ARROW_LEFT,
        Key::KEY_RIGHT => KeyCode::ARROW_RIGHT,
        Key::KEY_UP => KeyCode::ARROW_UP,
        Key::KEY_DOWN => KeyCode::ARROW_DOWN,
        Key::KEY_SPACE => KeyCode::SPACE,
        Key::KEY_ENTER => KeyCode::ENTER,
        Key::BTN_LEFT => KeyCode::MOUSE_PRIMARY,
        Key::KEY_P => KeyCode::LETTER_P,
        _ => KeyCode::UNKNOWN
    }
}

pub fn setup_evdev_watcher(mut device: Device, stop: Arc<AtomicBool>, tx: Sender<IncomingMessage>, screen_size: Rect) {
    thread::spawn(move || {
        let mut cx = 0.0;
        let mut cy = 0.0;
        loop {
            if stop.load(Ordering::Relaxed) == true {
                println!("keyboard thread stopping");
                break;
            }
            for ev in device.fetch_events().unwrap() {
                // println!("{:?}", ev);
                // info!("type {:?}", ev.event_type()); //type and kind are the same. kind is just nicer
                match ev.kind() {
                    InputEventKind::Key(key) => {
                        info!("evdev:key {} value {}",key.code(),ev.value());
                        //pressed is value=1
                        //repeat is value = 2
                        //released is value = 0
                        let keycode = linuxkernel_to_KeyCode(key.code());
                        let appcmd = match keycode {
                            KeyCode::MOUSE_PRIMARY => {
                                if ev.value() == 1 {
                                    APICommand::MouseDown(MouseDownEvent {
                                        app_id: Default::default(),
                                        window_id: Default::default(),
                                        original_timestamp: 0,
                                        button: MouseButton::Primary,
                                        x: cx as i32,
                                        y: cy as i32,
                                    })
                                } else {
                                    APICommand::MouseUp(MouseUpEvent {
                                        original_timestamp: 0,
                                        button: MouseButton::Primary,
                                        x: cx as i32,
                                        y: cy as i32,
                                    })
                                }
                            },
                            _ => {
                                if ev.value() == 1 {
                                    APICommand::KeyDown(KeyDownEvent {
                                        app_id: Default::default(),
                                        window_id: Default::default(),
                                        original_timestamp: 0,
                                        key: keycode,
                                    })
                                } else {
                                    APICommand::KeyUp(KeyUpEvent {
                                        app_id: Default::default(),
                                        window_id: Default::default(),
                                        original_timestamp: 0,
                                        key: keycode,
                                    })
                                }
                            }
                        };
                        let cmd = IncomingMessage {
                            source: Default::default(),
                            command: appcmd
                        };
                        tx.send(cmd).unwrap()
                    },
                    InputEventKind::RelAxis(rel) => {
                        // info!("mouse event {:?} {}",rel, ev.value());
                        let v = ev.value() as f32;
                        match rel {
                            RelativeAxisType::REL_X => cx += v,
                            RelativeAxisType::REL_Y => cy += v,
                            _ => {
                                warn!("unknown relative axis type");
                            }
                        }
                        // info!("cursor {},{}",cx, cy);
                        if cx < 0.0 {
                            cx = 0.0;
                        }
                        if cy < 0.0 {
                            cy = 0.0;
                        }
                        let cmd = IncomingMessage {
                            source: Default::default(),
                            command: APICommand::MouseMove(MouseMoveEvent{
                                original_timestamp: 0,
                                button: MouseButton::Primary,
                                x:cx as i32,
                                y:cy as i32
                            })
                        };
                        tx.send(cmd).unwrap()
                    },
                    InputEventKind::AbsAxis(abs) => {
                        // info!("abs event {:?} {:?}",ev.value(), abs);
                        let max_u16 = 32767 as f32;
                        let w = screen_size.w as f32;
                        let h = screen_size.h as f32;
                        let v = ev.value() as f32;
                        let mut was_y = false;
                        match abs {
                            AbsoluteAxisType::ABS_X => cx = v/max_u16*w,
                            AbsoluteAxisType::ABS_Y => {
                                cy = v/max_u16*h;
                                was_y = true
                            },
                            _ => {
                                warn!("unknown aboslute axis type")
                            }
                        }
                        // info!("cursor {} , {}",cx, cy);
                        let cmd = IncomingMessage {
                            source: Default::default(),
                            command: APICommand::MouseMove(MouseMoveEvent {
                                original_timestamp: 0,
                                button: MouseButton::Primary,
                                x: cx as i32,
                                y: cy as i32
                            }),
                        };
                        if was_y {
                            tx.send(cmd).unwrap();
                        }
                    },
                    _ => {}
                }
            }
        }
    });
}
