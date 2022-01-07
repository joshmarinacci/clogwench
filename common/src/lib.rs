use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ARGBColor {
    pub r:u8,
    pub g:u8,
    pub b:u8,
    pub a:u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DrawRectCommand {
    pub x:i32,
    pub y:i32,
    pub w:i32,
    pub h:i32,
    pub color:ARGBColor,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenWindowCommand {
    pub name:i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum APICommand {
    DrawRectCommand(DrawRectCommand),
    OpenWindowCommand(OpenWindowCommand),
}
