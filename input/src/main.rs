// cli/"tui" shared between the evtest examples
mod _pick_device;
use evdev::{Device, Key, EventType, InputEventKind};

fn find_keyboard() -> Option<evdev::Device> {
    let mut devices = evdev::enumerate().collect::<Vec<_>>();
    for (i, d) in devices.iter().enumerate() {
        if d.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
            // println!("found a keyboard");
            return devices.into_iter().nth(i);
            // let st = d.physical_path().unwrap().to_string();
            // return Some(d)
        }
        // println!("name:{} path:{} unique_name:{} id:{}",
        //     d.name().unwrap_or("Unnamed device"), 
        //     d.physical_path().unwrap_or("unknown path"),
        //     d.unique_name().unwrap_or("unknown uname"),
        //     d.input_id().vendor(), 
        // );
        // for typ in d.supported_events().iter() {
        //     println!("   type {:?}",typ);
        // }
        // if d.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
        //     println!("can emit an enter key");
        // }
    }
    None
}

fn print_all_devices() {
    let mut devices = evdev::enumerate().collect::<Vec<_>>();
    devices.reverse();
    for (i, d) in devices.iter().enumerate() {
        println!("name:{} path:{} unique_name:{} id:{}",
            d.name().unwrap_or("Unnamed device"), 
            d.physical_path().unwrap_or("unknown path"),
            d.unique_name().unwrap_or("unknown uname"),
            d.input_id().vendor(), 
        );
        for typ in d.supported_events().iter() {
            println!("   type {:?}",typ);
        }
        if d.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
            println!("can emit an enter key");
        }
    }
}

fn main() {
    println!("input devices");
    let mut keyboard = find_keyboard().expect("couldnt find the keyboard");
    println!("found the keyboard {}",keyboard);
//    let keybd = Device::open(keybd_path).expect("Couldn't open the keyboard.");
//    let mut d = _pick_device::pick_device();
    // let device = Device::open("/dev/input/event1").unwrap();
    // if device.supported_keys().map_or(false, |keys| keys.contains(Key::KEY_ENTER)) {
        // println!("are you prepared to ENTER the world of evdev?");
    // } else {
        // println!(":(");
    // }

    // println!("{}", d);
    println!("Events:");
    let mut go = true;
    loop {
        if !go {
            break;
        }
        for ev in keyboard.fetch_events().unwrap() {
            // println!("{:?}", ev);
            // println!("type {:?}", ev.event_type());
            if let InputEventKind::Key(key) = ev.kind() {
                println!("a key was pressed: {}",key.code());
                if key == Key::KEY_ESC {
                    println!("trying to escape");
                    go = false
                }
            }
        }
    }
}
