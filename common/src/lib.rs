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
pub struct DrawRectCommand {
    pub x:i32,
    pub y:i32,
    pub w:i32,
    pub h:i32,
}
