use std::fmt::Formatter;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use db::JObj;
use gfx::graphics::{ARGBColor, GFXBuffer, Rect, Size};
use crate::events::{KeyDownEvent, KeyUpEvent, MouseDownEvent};


pub mod client;
pub mod events;
pub mod generated;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloApp {

}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloAppResponse {
    pub app_id:Uuid,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppDisconnected {
    pub app_id:Uuid,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloWindowManager {

}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloWindowManagerResponse {
    pub wm_id:Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrawRectCommand {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub rect:Rect,
    pub color:ARGBColor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrawImageCommand {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub rect:Rect,
    pub buffer:GFXBuffer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenWindowCommand {
    pub window_type:String,
    pub window_title:String,
    pub bounds:Rect,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpenWindowResponse {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub window_type:String,
    pub window_title:String,
    pub bounds:Rect,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloseWindowResponse {
    pub app_id:Uuid,
    pub window_id:Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowResized {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub size:Size,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DBQueryClauseKind {
    equals,
    equalsi,
    substring,
    substringi,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBQueryClause {
    pub kind:DBQueryClauseKind,
    pub key:String,
    pub value:String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBQueryRequest {
    pub app_id:Uuid,
    pub query:Vec<DBQueryClause>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBQueryResponse {
    pub app_id:Uuid,
    pub success:bool,
    pub results: Vec<JObj>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBAddRequest {
    pub app_id:Uuid,
    pub object:JObj,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBAddResponse {
    pub app_id:Uuid,
    pub success:bool,
    pub object:JObj,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBUpdateRequest {
    pub app_id:Uuid,
    pub object:JObj,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBUpdateResponse {
    pub app_id:Uuid,
    pub success:bool,
    pub object:JObj,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBDeleteRequest {
    pub app_id:Uuid,
    pub object:JObj,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBDeleteResponse {
    pub app_id:Uuid,
    pub success:bool,
    pub object:JObj,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioPlayTrackRequest {
    pub app_id:Uuid,
    pub track:JObj,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioPlayTrackResponse {
    pub app_id:Uuid,
    pub success:bool,
    pub track:JObj,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioPauseTrackRequest {
    pub app_id:Uuid,
    pub track:JObj,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioPauseTrackResponse {
    pub app_id:Uuid,
    pub success:bool,
    pub track:JObj,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum APICommand {
    AppConnect(HelloApp),
    AppConnectResponse(HelloAppResponse),
    AppDisconnected(AppDisconnected),
    Debug(DebugMessage),

    WMConnect(HelloWindowManager),
    WMConnectResponse(HelloWindowManagerResponse),

    OpenWindowCommand(OpenWindowCommand),
    OpenWindowResponse(OpenWindowResponse),
    CloseWindowResponse(CloseWindowResponse),
    WindowResized(WindowResized),

    DrawRectCommand(DrawRectCommand),
    DrawImageCommand(DrawImageCommand),

    KeyDown(KeyDownEvent),
    KeyUp(KeyUpEvent),
    MouseDown(crate::events::MouseDownEvent),
    MouseMove(crate::events::MouseMoveEvent),
    MouseUp(crate::events::MouseUpEvent),

    DBQueryRequest(DBQueryRequest),
    DBQueryResponse(DBQueryResponse),
    DBAddRequest(DBAddRequest),
    DBAddResponse(DBAddResponse),
    DBUpdateRequest(DBUpdateRequest),
    DBUpdateResponse(DBUpdateResponse),
    DBDeleteRequest(DBDeleteRequest),
    DBDeleteResponse(DBDeleteResponse),

    AudioPlayTrackRequest(AudioPlayTrackRequest),
    AudioPlayTrackResponse(AudioPlayTrackResponse),
    AudioPauseTrackRequest(AudioPauseTrackRequest),
    AudioPauseTrackResponse(AudioPauseTrackResponse),

    SystemShutdown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IncomingMessage {
    pub source:Uuid,
    pub command:APICommand,
    pub trace:bool,
    pub timestamp_usec:u128,
}


pub const DEBUG_PORT:i32 = 3335;
pub const WINDOW_MANAGER_PORT:i32 = 3334;
pub const APP_MANAGER_PORT:i32 = 3333;



#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DebugMessage {
    HelloDebugger,
    HelloDebuggerResponse,
    ServerStarted,
    ServerStopped,
    WindowManagerConnected,
    WindowManagerDisconnected,
    AppConnected(String),
    AppDisconnected(String),
    WindowOpened(String),
    WindowClosed(String),
    BackgroundReceivedMouseEvent,
    WindowFocusChanged(String),
    RequestServerShutdown,
    AppLog(String),
    FakeMouseEvent(MouseDownEvent),
    ScreenCapture(Rect,String),
    ScreenCaptureResponse(),
}

#[test]
fn test_rect_intersect() {
    let r1 = Rect::from_ints(0,0,500,500);
    let r2 = Rect::from_ints(0,0,250,250);
    let r3 = r1.intersect(r2);
    println!("r3 {}",r3);
    assert_eq!(r3,Rect::from_ints(0,0,250,250));
}
