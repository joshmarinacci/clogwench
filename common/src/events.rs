use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::generated::KeyCode;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModifierState {
    pub shift:bool,
    pub ctrl:bool,
    pub alt:bool,
    pub meta:bool,
}

impl ModifierState {
    pub fn empty() -> ModifierState {
        ModifierState {
            shift:false,
            ctrl:false,
            alt:false,
            meta:false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyDownEvent {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub key:KeyCode,
    pub mods:ModifierState,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyUpEvent {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub key:KeyCode,
    pub mods:ModifierState,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MouseButton {
    Primary,
    Secondary
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseDownEvent {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub original_timestamp:u128,
    pub button:MouseButton,
    pub x:i32,
    pub y:i32,
}

impl MouseDownEvent {
    pub fn init_primary(x: i32, y: i32) -> MouseDownEvent {
        MouseDownEvent {
            app_id: Default::default(),
            window_id: Default::default(),
            original_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            button: MouseButton::Primary,
            x,
            y,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseMoveEvent {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub original_timestamp:u128,
    pub button:MouseButton,
    pub x:i32,
    pub y:i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MouseUpEvent {
    pub app_id:Uuid,
    pub window_id:Uuid,
    pub original_timestamp:u128,
    pub button:MouseButton,
    pub x:i32,
    pub y:i32,
}

