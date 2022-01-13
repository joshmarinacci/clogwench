mod surf;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use common::graphics::{ColorDepth, GFXBuffer};
use common::{APICommand, ARGBColor, HelloWindowManager, IncomingMessage, Point, Rect, BLACK};
use log::{debug, info, LevelFilter, log, warn};
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use structopt::StructOpt;
use framebuffer::{Framebuffer, KdMode, VarScreeninfo};
use crate::surf::Surf;

fn main() {
    let args:Cli = init_setup();
    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    let pth = "/dev/fb0";
    let mut fb = Framebuffer::new(pth).unwrap();
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    let mut surf:Surf = Surf::make(fb);
    // surf.sync();
    let cursor_image:GFXBuffer = GFXBuffer::from_png_file("../resources/cursor.png");
    info!("loaded the cursor image");
    let drawing_thread = make_drawing_thread(surf,stop.clone(),cursor_image);

    let timeout_handle = start_timeout(stop.clone(),args.timeout);
    timeout_handle.join().unwrap();

    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();

}

fn make_drawing_thread(mut surf: Surf,
                       stop: Arc<AtomicBool>,
                       cursor_image: GFXBuffer
) -> JoinHandle<()> {
    return thread::spawn(move ||{
        info!("render thread starting");
        let mut cursor:Point = Point::init(0,0);
        loop {
            let now = Instant::now();
            if stop.load(Ordering::Relaxed) == true { break; }
            surf.buf.clear(&BLACK);
            let bounds = Rect::from_ints(0, 0, 200, 200);
            surf.buf.fill_rect(bounds, ARGBColor::new_rgb(255, 0, 0));
            surf.copy_from(cursor.x, cursor.y, &cursor_image);
            surf.sync();
            info!("drawing {}ms",(now.elapsed().as_millis()));
        }
        info!("render thread stopping");
    });
}

fn start_timeout(stop: Arc<AtomicBool>, max_seconds:u32) -> JoinHandle<()> {
    return thread::spawn(move || {
        info!("timeout will end in {} seconds",max_seconds);
        let mut count = 0;
        loop {
            count = count + 1;
            if count > max_seconds {
                info!("timeout triggered");
                stop.store(true,Ordering::Relaxed);
            }
            thread::sleep(Duration::from_millis(1000));
            if stop.load(Ordering::Relaxed) == true { break; }
        }
    });
}


#[derive(StructOpt, Debug)]
#[structopt(name = "test-wm", about = "simulates receiving and sending linux-wm events")]
struct Cli {
    #[structopt(short, long)]
    debug:bool,
    #[structopt(short, long, default_value="60")]
    timeout:u32,
    #[structopt(long)]
    disable_graphics:bool,
}

fn init_setup() -> Cli {
    let args:Cli = Cli::from_args();
    let loglevel = if args.debug { LevelFilter::Debug } else { LevelFilter::Error };

    // create file appender with target file path
    let logfile = FileAppender::builder()
        .build("log/output.log").expect("error setting up file appender");
    println!("logging to log/output.log");

    // make a config
    let config = Config::builder()
        //add the file appender
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        //now make it
        .build(Root::builder()
            .appender("logfile") // why do we need to mention logfile again?
            .build(loglevel)).expect("error setting up log file");

    log4rs::init_config(config).expect("error initing config");

    info!("running with args {:?}",args);
    return args;
}
