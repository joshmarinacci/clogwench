use std::sync::mpsc::RecvError;
use common::{APICommand, BLACK, DrawRectCommand, HelloApp, OpenWindowCommand, Rect, WHITE};
use common::client::ClientConnection;
use common::events::KeyCode;
use uuid::Uuid;

fn redraw(client: &ClientConnection, appid: Uuid, winid: Uuid, bounds: Rect, px: i32, py:i32) {
    //draw background and wait
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        app_id:appid,
        window_id:winid,
        rect: Rect { x:0, y:0, w:bounds.w, h:bounds.h},
        color: WHITE,
    }));
    //draw player and wait
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        app_id:appid,
        window_id:winid,
        rect:Rect{ x:px, y:py, w:10, h:10},
        color: BLACK,
    }));
}
fn main() {
    let bounds = Rect::from_ints(50,50,300,300);
    let mut px = 50;
    let mut py = 50;
    let mut appid = Uuid::new_v4();

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
        bounds: bounds,
        }));
    let mut winid = Uuid::new_v4();
    match resp2 {
        Ok(APICommand::OpenWindowResponse(wininfo)) => {
            winid = wininfo.window_id
        }
        _ => {
            panic!("error. response should have been from the app connect")
        }
    }
    redraw(&client, appid, winid, bounds, px, py);

    for cmd in &client.rx {
        println!("got an event {:?}",cmd);
        match cmd {
            APICommand::KeyDown(kd) => {
                println!("got a keydown event");
                match kd.key {
                    KeyCode::ARROW_RIGHT => px += 1,
                    KeyCode::ARROW_LEFT => px -= 1,
                    KeyCode::ARROW_UP => py -= 1,
                    KeyCode::ARROW_DOWN => py += 1,
                    _ => {}
                }
                redraw(&client, appid, winid, bounds, px, py);
            }
            _ => {}
        }
    }

}
