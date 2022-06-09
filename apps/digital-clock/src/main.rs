use std::io::{prelude::*};
use std::net::TcpStream;
use std::sync::mpsc::RecvError;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use log::{info, LevelFilter, set_logger};
use uuid::Uuid;
use common::{APICommand, DebugMessage, DrawImageCommand, DrawRectCommand, HelloApp, OpenWindowCommand, Rect, WHITE};
use common::client::ClientConnection;
use common::graphics::GFXBuffer;
use cool_logger::CoolLogger;


static COOL_LOGGER:CoolLogger = CoolLogger;

fn main()  {
    set_logger(&COOL_LOGGER).map(|()|log::set_max_level(LevelFilter::Info));
    let client = ClientConnection::init().expect("Can't connect to the central");

    let mut appid = Uuid::new_v4();
    if let Ok(APICommand::AppConnectResponse(appinfo)) = client.send_and_wait(APICommand::AppConnect(HelloApp{})) {
        appid = appinfo.app_id;
    } else {
        panic!("error. response should have been from the app connect")
    }

    let bounds = Rect::from_ints(50,50,300,300);
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


    let sprite_sheet = GFXBuffer::from_png_file("./digital_clock@1.png");



    loop {
        redraw(&client,appid,winid,bounds, &sprite_sheet);
        for cmd in client.rx.try_iter() {
            info!("got an event {:?}",cmd);
            match cmd {
                APICommand::SystemShutdown => {
                    info!("CLIENT app:  system is shutting down. bye!");
                    client.send(APICommand::Debug(DebugMessage::AppLog(String::from("got-shutdown"))));
                    break;
                }
                _ => {}
            }
        }
        sleep(Duration::from_secs(1));
    }
    info!("CLIENT APP ending");
}

fn redraw(client: &ClientConnection, appid: Uuid, winid: Uuid, bounds: Rect, sprite_sheet: &GFXBuffer) {
    //draw background
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        app_id:appid,
        window_id:winid,
        rect: Rect { x:0, y:0, w:bounds.w, h:bounds.h},
        color: WHITE,
    }));

    let offsets = [
        0, 27, 45, 71, 96, //0 - 4
        119,144,170,192,216, // 5 - 9
        242,256]; // : and ending

    let now = chrono::offset::Local::now();
    let hr_str = now.format("%H:%M:%S").to_string();
    // println!("the hour is {}",hr_str);
    let mut chx = 0;
    for letter in hr_str.chars() {
        let n = match letter {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            ':' => 10,
            _ => 0,
        };
        let sx1 = offsets[n];
        let sx2 = offsets[n+1];
        let imw = sx2 - sx1;
        // println!("letter {} is index {} - {}  x={} w={}",letter,n,n+1, sx1, imw);
        client.send(APICommand::DrawImageCommand(DrawImageCommand {
            app_id: appid,
            window_id: winid,
            rect: Rect::from_ints(chx, 30, imw, 32),
            buffer: sprite_sheet.sub_rect(Rect::from_ints(sx1, 0, imw, 32)),
        }));
        chx += imw;
        chx += 10;
    }

}
