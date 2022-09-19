

use std::sync::{Arc, mpsc};
use std::sync::atomic::AtomicBool;
use plat::{make_plat, Plat};
use common::graphics::draw_test_pattern;
use common::graphics::GFXBuffer;
use common::graphics::PixelLayout;
use common::{ARGBColor, BLACK, IncomingMessage, Point, WHITE};
use common::font::load_font_from_json;

fn main() -> Result<(),String> {
    let w = 320;
    let h = 240;
    let scale = 2;
    let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let (tx_in, mut rx_in) = mpsc::channel::<IncomingMessage>();

    let mut plat = make_plat(stop.clone(), tx_in.clone(), w, h,scale).unwrap();
    let bds = plat.get_screen_bounds();
    let mut background = GFXBuffer::new(bds.w as u32, bds.h as u32, &plat.get_preferred_pixel_layout());
    plat.register_image2(&background);
    let font = load_font_from_json("../resources/default-font.json").unwrap();


    let mut text_buffer = GFXBuffer::new(100, 20, &PixelLayout::ARGB());
    plat.register_image2(&text_buffer);
    text_buffer.clear(&BLACK);

    let mut tick = 0;
    while tick < 60*5 {
        tick += 1;
        plat.service_input();
        plat.clear();
        background.clear(&ARGBColor::new_rgb(120, 128, 128));
        plat.draw_image(&Point::init(0, 0), &background.bounds(), &background);
        font.draw_text_at(&mut text_buffer,"title bar!",0,10,&WHITE);

        plat.draw_image(&Point::init(100,100), &text_buffer.bounds(),
        &text_buffer);
        plat.service_loop();
    }
    plat.shutdown();

    Ok(())

}
