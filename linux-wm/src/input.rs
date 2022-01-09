use evdev::{AbsoluteAxisType, Device, EventType, InputEventKind, Key, RelativeAxisType};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use common::{APICommand, IncomingMessage};
use common::events::*;

use std::thread;
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
        _ => KeyCode::UNKNOWN
    }
}

pub fn setup_evdev_watcher(mut device: Device, stop: Arc<AtomicBool>, tx: Sender<IncomingMessage>) {
    thread::spawn(move || {
        let mut cx = 0;
        let mut cy = 0;
        loop {
            if stop.load(Ordering::Relaxed) == true {
                println!("keyboard thread stopping");
                break;
            }
            for ev in device.fetch_events().unwrap() {
                // println!("{:?}", ev);
                println!("type {:?}", ev.event_type());
                match ev.kind() {
                    InputEventKind::Key(key) => {
                        println!("   evdev:key {}",key.code());
                        let cmd = IncomingMessage {
                            source: Default::default(),
                            command: APICommand::KeyDown(KeyDownEvent{
                                app_id: Default::default(),
                                window_id: Default::default(),
                                original_timestamp: 0,
                                key:linuxkernel_to_KeyCode(key.code()),
                            })
                        };
                        tx.send(cmd).unwrap()
                    },
                    InputEventKind::RelAxis(rel) => {
                        println!("mouse event {:?} {}",rel, ev.value());
                        match rel {
                            RelativeAxisType::REL_X => cx += ev.value(),
                            RelativeAxisType::REL_Y => cy += ev.value(),
                            _ => {
                                println!("unknown relative axis type");
                            }
                        }
                        println!("cursor {} , {}",cx, cy);
                        let cmd = IncomingMessage {
                            source: Default::default(),
                            command: APICommand::MouseMove(MouseMoveEvent{
                                // app_id: Default::default(),
                                // window_id: Default::default(),
                                original_timestamp: 0,
                                button: MouseButton::Primary,
                                x:cx,
                                y:cy
                            })
                        };
                        // let cmd = APICommand::MouseMove(MouseMoveEvent{
                        //     original_timestamp:0,
                        //     button:MouseButton::Primary,
                        //     x:cx,
                        //     y:cy
                        // });
                        tx.send(cmd).unwrap()
                    },
                    InputEventKind::AbsAxis(abs) => {
                        println!("abs event {:?} {:?}",ev.value(), abs);
                        // match abs {
                        //     AbsoluteAxisType::ABS_X => cx = ev.value()/10,
                        //     AbsoluteAxisType::ABS_Y => cy = ev.value()/10,
                        //     _ => {
                        //         println!("unknown aboslute axis type")
                        //     }
                        // }
                        // let cmd = APICommand::MouseMove(MouseMoveEvent{
                        //     original_timestamp:0,
                        //     button:MouseButton::Primary,
                        //     x:cx,
                        //     y:cy
                        // });
                        // tx.send(cmd).unwrap()
                        //stop.store(true,Ordering::Relaxed);
                    },
                    _ => {}
                }
            }
        }
    });
}
