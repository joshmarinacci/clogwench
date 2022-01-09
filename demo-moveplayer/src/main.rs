use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{IntoIter, Iter, Receiver, Sender, TryIter};
use std::thread;
use serde::Deserialize;
use common::{APICommand, ARGBColor, BLACK, DrawRectCommand, OpenWindowCommand, Rect, WHITE};
use common::client::client::ClientConnection;

fn redraw(client: &ClientConnection, x: i32, y: i32, w:i32, h:i32) {
    //draw background and wait
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        x:0, y:0, w, h, color: WHITE,
    }));
    //draw player and wait
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        x, y, w:10, h:10, color: BLACK,
    }));
}
fn main() {
    let w = 100;
    let h = 100;
    let mut x = 50;
    let mut y = 50;

    let client = ClientConnection::init().expect("Can't connect to the server");
    //open window and wait
    client.send(APICommand::OpenWindowCommand(OpenWindowCommand{
        name: 0,
        window_type: String::from("plain"),
        bounds: Rect::from_ints(x,y,w,h),
        }));
    redraw(&client,x,y,w,h);

    for cmd in &client.rx {
        println!("got an event {:?}",cmd);
        match cmd {
            APICommand::KeyDown(kd) => {
                println!("got a keydown event");
                x += 1;
                redraw(&client,x,y,w,h)
            }
            _ => {}
        }
    }

}
