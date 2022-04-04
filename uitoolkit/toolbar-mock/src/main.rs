mod core;
mod components;
mod v2try;

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
use crate::components::{HSpacer, Label};
use crate::core::{ActionEvent, DrawingSurface, repaint, start_loop, UIChild, UIView};

static COOL_LOGGER:CoolLogger = CoolLogger;


fn main() {
    set_logger(&COOL_LOGGER).map(|()|log::set_max_level(LevelFilter::Info));

    info!("toolbar app starting and connecting");
    let client = ClientConnection::init().expect("Can't connect to the central");

    let (appid,winid) = open_window(&client);
    let font = load_font_from_json("../../resources/default-font.json").unwrap();

    let mut hbox:HBox = HBox::make();
    let mut button1: ActionButton = ActionButton::make();
    button1._caption = "Prev".to_string();
    hbox.add(button_to_ui_child(button1));
    let mut button2: ActionButton = ActionButton::make();
    button2._caption = "Play".to_string();
    hbox.add(button_to_ui_child(button2));
    let mut button3: ActionButton = ActionButton::make();
    button3._caption = "Next".to_string();
    button3._dispatcher.add_event_listener(|event:&ActionEvent| {
        println!("an action happened. sweet!");
    });
    hbox.add(button_to_ui_child(button3));

    let mut spacer = HSpacer::make();
    hbox.add(hspacer_to_ui_child(spacer));

    let mut label4 = Label::make();
    label4._caption = "cool label".to_string();
    hbox.add(label_to_ui_child(label4));

    let mut spacer = HSpacer::make();
    hbox.add(hspacer_to_ui_child(spacer));


    let mut button5 = ActionButton::make();
    button5._caption = "add".to_string();
    hbox.add(button_to_ui_child(button5));

    let mut surf:DrawingSurface = DrawingSurface::init(appid,winid,font,client);
    let root:UIChild = Rc::new(RefCell::new(hbox));
    repaint(&mut surf, root.clone());
    start_loop(&mut surf, root.clone());
    println!("CLIENT APP ending");
}

fn hspacer_to_ui_child(child: HSpacer) -> UIChild {
    Rc::new(RefCell::new(child))
}
fn button_to_ui_child(child: ActionButton) -> UIChild {
    Rc::new(RefCell::new(child))
}
fn label_to_ui_child(child: Label) -> UIChild {
    Rc::new(RefCell::new(child))
}

fn open_window(client: &ClientConnection) -> (Uuid, Uuid) {
    let bounds = Rect::from_ints(50,50,300,300);
    //open window and wait
    let result_1: Result<APICommand, RecvError> = client.send_and_wait(APICommand::AppConnect(HelloApp{}));
    let mut appid = Uuid::new_v4();
    match result_1 {
        Ok(APICommand::AppConnectResponse(appinfo)) => {
            appid = appinfo.app_id
        }
        _ => {
            panic!("error. response should have been from the app connect")
        }
    }
    let result_2: Result<APICommand, RecvError> = client.send_and_wait(APICommand::OpenWindowCommand(OpenWindowCommand{
        window_type: String::from("plain"),
        bounds: bounds,
    }));
    let mut winid = Uuid::new_v4();
    match result_2 {
        Ok(APICommand::OpenWindowResponse(wininfo)) => {
            winid = wininfo.window_id
        }
        _ => {
            panic!("error. response should have been from the app connect")
        }
    };


    return(appid,winid)
}
