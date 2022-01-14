use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KeyCode {
    RESERVED,
    ESC,
    UNKNOWN,
    ARROW_LEFT,
    ARROW_RIGHT,
    ARROW_UP,
    ARROW_DOWN,
    SPACE,
    ENTER,

    MOUSE_PRIMARY,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyDownEvent {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub original_timestamp:i64,
    pub key:KeyCode
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyUpEvent {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub original_timestamp:i64,
    pub key:KeyCode
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MouseButton {
    Primary,
    Secondary
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseDownEvent {
    pub original_timestamp:i64,
    pub button:MouseButton,
    pub x:i32,
    pub y:i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseMoveEvent {
    pub original_timestamp:i64,
    pub button:MouseButton,
    pub x:i32,
    pub y:i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseUpEvent {
    pub original_timestamp:i64,
    pub button:MouseButton,
    pub x:i32,
    pub y:i32,
}

