use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum KeyCode {
    RESERVED,
    ESC,
    UNKNOWN,
    ARROW_LEFT,
    ARROW_RIGHT,
    SPACE,
    ENTER,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyDownEvent {
    pub original_timestamp:i64,
    pub key:KeyCode
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyUpEvent {
    pub original_timestamp:i64,
    pub key:KeyCode
}


#[derive(Serialize, Deserialize, Debug)]
pub enum MouseButton {
    Primary,
    Secondary
}


#[derive(Serialize, Deserialize, Debug)]
pub struct MouseDownEvent {
    pub original_timestamp:i64,
    pub button:MouseButton,
    pub x:i32,
    pub y:i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MouseMoveEvent {
    pub original_timestamp:i64,
    pub button:MouseButton,
    pub x:i32,
    pub y:i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MouseUpEvent {
    pub original_timestamp:i64,
    pub button:MouseButton,
    pub x:i32,
    pub y:i32,
}

