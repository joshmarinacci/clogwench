use common::Rect;
use plat::Plat;

fn main() {
    let mut plat = Plat::init().unwrap();
    println!("Made a plat");

    let bounds:Rect = plat.get_screen_bounds();
    println!("screen bounds are {:?}",bounds);

    let mut count = 0;
    loop {
        plat.service_loop();
        count += 1;
        // println!("count {}",count);
        if count > 500 {
            break;
        }
    }
    plat.shutdown();
}
