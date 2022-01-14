use common::{APICommand, IncomingMessage};
use common::events::{KeyCode, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use core::default::Default;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use log::info;

pub fn send_fake_mouse(stop: Arc<AtomicBool>, sender: Sender<IncomingMessage>) {
    thread::spawn({
        info!("starting fake mouse events");
        move || {
            while stop.load(Ordering::Relaxed) == false {
                send_mousedown(&sender,55,55);
                sleep(1000);

                //drag over 5 spots to the right
                for off in 0..5 {
                    send_mousedrag(&sender,55+off*10,55);
                    sleep(1000);
                }

                send_mouseup(&sender,55+4*10,55);
                sleep(1000);
                stop.store(true,Ordering::Relaxed);
            }
        }
    });

}

fn send_mousedrag(sender: &Sender<IncomingMessage>, x: i32, y: i32) {
    sender.send(IncomingMessage{
        source: Default::default(),
        command:APICommand::MouseMove(MouseMoveEvent{
            original_timestamp: 0,
            button: MouseButton::Primary,
            x,
            y,
        })
    }).unwrap();
}

fn send_mouseup(sender: &Sender<IncomingMessage>, x: i32, y: i32) {
    //release
    let command: APICommand = APICommand::MouseUp(MouseUpEvent {
        original_timestamp: 0,
        button: MouseButton::Primary,
        x,
        y,
    });
    sender.send(IncomingMessage{ source: Default::default(),  command }).unwrap();
}

fn sleep(ms: u64) {
    thread::sleep(Duration::from_millis(ms))
}

fn send_mousedown(sender: &Sender<IncomingMessage>, x: i32, y: i32) {
    let command: APICommand = APICommand::MouseDown(MouseDownEvent {
        original_timestamp: 0,
        button: MouseButton::Primary,
        x: x,
        y: y,
    });
    sender.send(IncomingMessage{
        source: Default::default(),
        command
    }).unwrap();
}

pub fn send_fake_keyboard(stop: Arc<AtomicBool>, sender: Sender<IncomingMessage>) {
    thread::spawn({
        move || {
            loop {
                if stop.load(Ordering::Relaxed) { break; }
                let command: APICommand = APICommand::KeyDown(KeyDownEvent {
                    app_id: Default::default(),
                    window_id: Default::default(),
                    original_timestamp: 0,
                    key: KeyCode::ARROW_RIGHT
                });
                sender.send(IncomingMessage{
                    source: Default::default(),
                    command
                }).unwrap();
                thread::sleep(Duration::from_millis(1000));
            }
        }
    });

}
