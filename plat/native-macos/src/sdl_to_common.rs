
use sdl2::keyboard::Keycode;
use common::generated::KeyCode;

//hi. cool!
pub fn sdl_to_common(kc: Keycode) -> KeyCode {
    println!("converting SDL {}",kc);
    let code = match kc {
        Keycode::A => KeyCode::LETTER_A,
        Keycode::B => KeyCode::LETTER_B,
        Keycode::C => KeyCode::LETTER_C,
        Keycode::D => KeyCode::LETTER_D,
        Keycode::E => KeyCode::LETTER_E,
        Keycode::F => KeyCode::LETTER_F,
        Keycode::G => KeyCode::LETTER_G,
        Keycode::H => KeyCode::LETTER_H,
        Keycode::I => KeyCode::LETTER_I,
        Keycode::J => KeyCode::LETTER_J,
        Keycode::K => KeyCode::LETTER_K,
        Keycode::L => KeyCode::LETTER_L,
        Keycode::M => KeyCode::LETTER_M,
        Keycode::N => KeyCode::LETTER_N,
        Keycode::O => KeyCode::LETTER_O,
        Keycode::P => KeyCode::LETTER_P,
        Keycode::Q => KeyCode::LETTER_Q,
        Keycode::R => KeyCode::LETTER_R,
        Keycode::S => KeyCode::LETTER_S,
        Keycode::T => KeyCode::LETTER_T,
        Keycode::U => KeyCode::LETTER_U,
        Keycode::V => KeyCode::LETTER_V,
        Keycode::W => KeyCode::LETTER_W,
        Keycode::X => KeyCode::LETTER_X,
        Keycode::Y => KeyCode::LETTER_Y,
        Keycode::Z => KeyCode::LETTER_Z,   
        Keycode::Num0 => KeyCode::DIGIT_0,
        Keycode::Num1 => KeyCode::DIGIT_1,
        Keycode::Num2 => KeyCode::DIGIT_2,
        Keycode::Num3 => KeyCode::DIGIT_3,
        Keycode::Num4 => KeyCode::DIGIT_4,
        Keycode::Num5 => KeyCode::DIGIT_5,
        Keycode::Num6 => KeyCode::DIGIT_6,
        Keycode::Num7 => KeyCode::DIGIT_7,
        Keycode::Num8 => KeyCode::DIGIT_8,
        Keycode::Num9 => KeyCode::DIGIT_9,   
        Keycode::Left => KeyCode::ARROW_LEFT,
        Keycode::Right => KeyCode::ARROW_RIGHT,
        Keycode::Up => KeyCode::ARROW_UP,
        Keycode::Down => KeyCode::ARROW_DOWN,
        Keycode::LShift => KeyCode::SHIFT_LEFT,
        Keycode::RShift => KeyCode::SHIFT_RIGHT,
        Keycode::LCtrl => KeyCode::CONTROL_LEFT,
        Keycode::RCtrl => KeyCode::CONTROL_RIGHT,
        Keycode::LAlt => KeyCode::ALT_LEFT,
        Keycode::RAlt => KeyCode::ALT_RIGHT,
        Keycode::LGui => KeyCode::META_LEFT,
        Keycode::RGui => KeyCode::META_RIGHT,
        Keycode::Backspace => KeyCode::BACKSPACE,
        Keycode::Delete => KeyCode::DELETE,
        Keycode::Return => KeyCode::ENTER,
        Keycode::Escape => KeyCode::ESCAPE,
        Keycode::Tab => KeyCode::TAB,   
        _ => {
            KeyCode::UNKNOWN
        }
    };
    println!("to code {:?}",code);
    return code
}
