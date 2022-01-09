use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{IntoIter, Iter, Receiver, RecvError, Sender, TryIter};
use std::thread;
use serde::Deserialize;
use common::{APICommand, ARGBColor, BLACK, DrawRectCommand, HelloApp, OpenWindowCommand, Rect, WHITE};
use common::client::ClientConnection;
use common::events::KeyCode;
use uuid::Uuid;

fn redraw(client: &ClientConnection, x: i32, y: i32, w:i32, h:i32) {
    //draw background and wait
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        rect: Rect { x:0, y:0, w, h},
        color: WHITE,
        window_id: Default::default()
    }));
    //draw player and wait
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        rect:Rect{ x, y, w:10, h:10},
        color: BLACK,
        window_id: Default::default()
    }));
}
fn main() {
    let w = 100;
    let h = 100;
    let mut x = 50;
    let mut y = 50;
    let mut appid = Uuid::new_v4();
    let mut winid = Uuid::new_v4();

    let client = ClientConnection::init().expect("Can't connect to the linux-wm");
    //open window and wait
    let resp: Result<APICommand, RecvError> = client.send_and_wait(APICommand::AppConnect(HelloApp{}));
    match resp {
        Ok(APICommand::AppConnectResponse(appinfo)) => {
            appid = appinfo.app_id
        }
        _ => {
            panic!("error. response should have been from the app connect")
        }
    }
    let resp2: Result<APICommand, RecvError> = client.send_and_wait(APICommand::OpenWindowCommand(OpenWindowCommand{
        window_type: String::from("plain"),
        bounds: Rect::from_ints(x,y,w,h),
        }));
    match resp2 {
        Ok(APICommand::OpenWindowResponse(wininfo)) => {
            winid = wininfo.window_id
        }
        _ => {
            panic!("error. response should have been from the app connect")
        }
    }
    redraw(&client,x,y,w,h);

    for cmd in &client.rx {
        println!("got an event {:?}",cmd);
        match cmd {
            APICommand::KeyDown(kd) => {
                println!("got a keydown event");
                match kd.key {
                    KeyCode::ARROW_RIGHT => x += 1,
                    KeyCode::ARROW_LEFT => x -= 1,
                    _ => {}
                }
                redraw(&client,x,y,w,h)
            }
            _ => {}
        }
    }

}
