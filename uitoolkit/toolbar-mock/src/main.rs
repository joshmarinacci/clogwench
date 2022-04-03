mod core;
mod components;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::RecvError;
use log::{info, LevelFilter, set_logger};
use uuid::Uuid;
use common::{APICommand, ARGBColor, BLACK, DrawImageCommand, DrawRectCommand, HelloApp, OpenWindowCommand, Padding, Point, Rect, Size, WHITE};
use common::client::ClientConnection;
use common::font::{load_font_from_json};
use components::{ActionButton, HBox};
use cool_logger::CoolLogger;
use crate::core::{ActionEvent, DrawingSurface, UIView};

static COOL_LOGGER:CoolLogger = CoolLogger;


fn main() {
    set_logger(&COOL_LOGGER).map(|()|log::set_max_level(LevelFilter::Info));

    info!("toolbar app starting and connecting");
    let bounds = Rect::from_ints(50,50,300,300);
    // let mut px = 50;
    // let mut py = 50;
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

    let font = load_font_from_json("../../resources/default-font.json").unwrap();

    let mut hbox:HBox = HBox::make();
    let mut button1: ActionButton = ActionButton::make();
    button1._caption = "abc".to_string();
    hbox.add(button1);
    let mut button2: ActionButton = ActionButton::make();
    button2._caption = "def".to_string();
    hbox.add(button2);
    let mut button3: ActionButton = ActionButton::make();
    button3._caption = "ghijklm".to_string();
    button3._dispatcher.add_event_listener(|event:&ActionEvent| {
        println!("an action happened. sweet!");
    });
    hbox.add(button3);


    let mut surf:DrawingSurface = DrawingSurface::init(appid,winid,font,client,hbox);
    surf.repaint();
    surf.start_loop();
    println!("CLIENT APP ending");
}
