use cool_logger::CoolLogger;
use log::{info, LevelFilter, set_logger};
use std::sync::mpsc::RecvError;
use common::{APICommand, ARGBColor, BLACK, DebugMessage, DrawImageCommand, DrawRectCommand, HelloApp, OpenWindowCommand, Rect, WHITE};
use common::client::ClientConnection;
use common::events::KeyCode;
use uuid::Uuid;
use common::font::load_font_from_json;
use common::graphics::{GFXBuffer, PixelLayout};

static COOL_LOGGER:CoolLogger = CoolLogger;

fn redraw(client: &ClientConnection, appid: Uuid, winid: Uuid, bounds: Rect, px: i32, py:i32, pattern: &GFXBuffer, textbuff: &GFXBuffer) {
    //draw background
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        app_id:appid,
        window_id:winid,
        rect: Rect { x:0, y:0, w:bounds.w, h:bounds.h},
        color: WHITE,
    }));
    //draw player
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        app_id:appid,
        window_id:winid,
        rect:Rect{ x:px, y:py, w:10, h:10},
        color: ARGBColor::new_rgb(0,200,255),
    }));

    //draw buffer
    client.send(APICommand::DrawImageCommand(DrawImageCommand{
        app_id:appid,
        window_id:winid,
        rect:Rect{ x:40, y:40, w:50, h:50},
        buffer: pattern.clone(),
    }));

    client.send(APICommand::DrawImageCommand(DrawImageCommand{
        app_id:appid,
        window_id:winid,
        rect:Rect{ x:0, y:100, w: textbuff.width as i32, h: textbuff.height as i32 },
        buffer: textbuff.clone(),
    }));
}
fn main() {
    set_logger(&COOL_LOGGER).map(|()|log::set_max_level(LevelFilter::Info));

    let mut pattern_buffer = GFXBuffer::new(2,2, &PixelLayout::ARGB());
    pattern_buffer.set_pixel_vec_argb(0,0, &WHITE.to_argb_vec());
    pattern_buffer.set_pixel_vec_argb(1,0, &BLACK.to_argb_vec());
    pattern_buffer.set_pixel_vec_argb(0,1, &BLACK.to_argb_vec());
    pattern_buffer.set_pixel_vec_argb(1,1, &WHITE.to_argb_vec());


    let mut text_buffer = GFXBuffer::new(100, 20, &PixelLayout::ARGB());
    text_buffer.clear(&BLACK);
    let font = load_font_from_json("../../resources/default-font.json").unwrap();
    font.draw_text_at(&mut text_buffer,"Echo Bot Here!",0,10,&ARGBColor::new_rgb(0,255,0));

    info!("echo app starting and connecting");
    let mut bounds = Rect::from_ints(50,50,300,300);
    let mut px = 50;
    let mut py = 50;
    let mut appid = Uuid::new_v4();

    let client = ClientConnection::init().expect("Can't connect to the central");
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
        window_title: String::from("Echo")
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
    redraw(&client, appid, winid, bounds, px, py, &pattern_buffer, &text_buffer);

    for cmd in &client.rx {
        info!("got an event {:?}",cmd);
        match cmd {
            APICommand::KeyDown(kd) => {
                info!("got a keydown event");
                match kd.key {
                    KeyCode::ARROW_RIGHT => px += 1,
                    KeyCode::ARROW_LEFT => px -= 1,
                    KeyCode::ARROW_UP => py -= 1,
                    KeyCode::ARROW_DOWN => py += 1,
                    _ => {}
                }
                redraw(&client, appid, winid, bounds, px, py, &pattern_buffer, &text_buffer);
                client.send(APICommand::Debug(DebugMessage::AppLog(String::from("got-keyboard-event"))));
            }
            APICommand::MouseDown(md) => {
                client.send(APICommand::Debug(DebugMessage::AppLog(String::from("got-mouse-event"))));
            }
            APICommand::CloseWindowResponse(cmd) => {
                info!("onlyhave one window. shutting down the whole app");
                break;
            }
            APICommand::SystemShutdown => {
                info!("CLIENT app:  system is shutting down. bye!");
                client.send(APICommand::Debug(DebugMessage::AppLog(String::from("got-shutdown"))));
                break;
            }
            APICommand::WindowResized(evt) => {
                info!("got a window resize event {:?}", evt);
                bounds.set_size(evt.size);
                redraw(&client, appid, winid, bounds, px, py, &pattern_buffer, &text_buffer);
            }
            _ => {}
        }
    }
    println!("CLIENT APP ending");

}
