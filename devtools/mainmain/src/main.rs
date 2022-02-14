use structopt::StructOpt;
use std::{env, thread};
use std::fs::File;
use std::io::BufWriter;
use std::net::TcpStream;
use std::process::Command;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log::{debug, error, info, LevelFilter};
use uuid::Uuid;
use common::graphics::{GFXBuffer, PixelLayout};
use common::{APICommand, ARGBColor, BLACK, IncomingMessage, Point, Rect, WHITE};
use common::APICommand::KeyDown;
use common::events::{KeyCode, KeyDownEvent};
use common::graphics::ColorDepth::CD32;
use common_wm::{FOCUSED_TITLEBAR_COLOR, FOCUSED_WINDOW_COLOR, InputGesture, NoOpGesture, OutgoingMessage, start_wm_network_connection, TITLEBAR_COLOR, WINDOW_BORDER_WIDTH, WINDOW_COLOR, WindowDragGesture, WindowManagerState};
use plat::{make_plat, Plat};

fn main() -> std::io::Result<()>{
    //initial setup
    let args:Cli = init_setup();
    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    let watchdog = make_watchdog(stop.clone());

    let tv:u32 = 0xFF;
    let tv_b = u32::from_be(tv);
    let tv_l = u32::from_le(tv);
    info!("normal {} big {} le{}",tv,tv_b,tv_l);

    //load the cursor image
    let cwd = env::current_dir()?;
    info!("cwd is {}", cwd.display());

    // let mut network_stream:Option<TcpStream> = None;
    //create empty channel first
    let (mut internal_message_sender,
        mut internal_message_receiver) = mpsc::channel::<IncomingMessage>();
    let (mut external_message_sender,
        mut external_message_receiver) = mpsc::channel::<OutgoingMessage>();

    //start the central server
    if args.start_server {
        info!("starting the central server");
        start_central_server(stop.clone());
        sleep(1000);
    }

    //connect to the central server
    if !args.disable_network {
        info!("connecting to the central server");
        //open network connection
        let conn = start_wm_network_connection(stop.clone(), internal_message_sender.clone())
            .expect("error connecting to the central server");
        // network_stream = Option::from(conn.stream);
        // internal_message_sender = conn.tx_in;
        external_message_sender = conn.tx_out;
        info!("fully connected to the network now");
    } else {
        info!("skipping the network connection");
    }




    //start test app
    // if args.start_app1 {
    //     start_test_app(stop.clone());
    // }


    let mut test_pattern = GFXBuffer::new(CD32(), 64, 64, PixelLayout::ARGB());
    common::graphics::draw_test_pattern(&mut test_pattern);
    //make the platform specific graphics
    let mut plat = make_plat(stop.clone(), internal_message_sender.clone()).unwrap();
    info!("Made a plat");
    plat.register_image2(&test_pattern);

    let cursor_image:GFXBuffer = GFXBuffer::from_png_file("../resources/cursor.png");
    plat.register_image2(&cursor_image);
    //setup the window manager state
    let mut state = WindowManagerState::init();
    {
        // preload a fake app and window
        let fake_app_1 = Uuid::new_v4();
        state.add_app(fake_app_1);
        let winid1 = state.add_window(fake_app_1, Uuid::new_v4(), &Rect::from_ints(400, 50, 100, 200));
        let win1 = state.lookup_window(winid1).unwrap();
        win1.backbuffer.clear(&WHITE);
        win1.backbuffer.fill_rect(Rect::from_ints(20,20,20,20), &ARGBColor::new_rgb(0, 255, 0));
        plat.register_image2(&win1.backbuffer);

        // let fake_app_2 = Uuid::new_v4();
        // state.add_app(fake_app_2);
        // state.add_window(fake_app_2, Uuid::new_v4(), &Rect::from_ints(250, 50, 100, 200));
    }


    let bounds:Rect = plat.get_screen_bounds();
    println!("screen bounds are {:?}",bounds);
    let mut cursor:Point = Point::init(0,0);

    // let mut count = 0;
    let mut gesture = Box::new(NoOpGesture::init()) as Box<dyn InputGesture>;
    loop {
        if stop.load(Ordering::Relaxed)==true {
            break;
        }
        plat.service_input();
        // info!("checking for incoming events");
        for cmd in internal_message_receiver.try_iter() {
            // info!("incoming {:?}",cmd);
            if stop.load(Ordering::Relaxed) == true { break; }
            match cmd.command {
                APICommand::AppConnectResponse(res) => {
                    info!("app connected");
                    state.add_app(res.app_id);
                }
                APICommand::OpenWindowResponse(ow) => {
                    info!("window opened");
                    let winid = state.add_window(ow.app_id, ow.window_id, &ow.bounds);
                    if let Some(win) = state.lookup_window(winid) {
                        plat.register_image2(&win.backbuffer);
                    }
                    // state.set_focused_window(ow.window_id);
                }
                APICommand::DrawRectCommand(dr) => {
                    info!("draw rect command");
                    if let Some(mut win) = state.lookup_window(dr.window_id) {
                        info!("drawing to window {} {:?} {:?}",win.id, dr.rect, dr.color);
                        win.backbuffer.fill_rect(dr.rect, &dr.color);
                    }
                }
                APICommand::KeyDown(kd) => {
                    info!("key down {:?}",kd.key);
                    match kd.key {
                        KeyCode::ESC => {
                            stop.store(true, Ordering::Relaxed);
                        },
                        KeyCode::LETTER_P => {
                            info!("doing a screencapture request");
                            capture_screen(&state,plat.get_screen_bounds(),"screencap.png");
                        }
                        _ => {
                            info!("key down");
                            //send key to the currently focused window
                            if let Some(winid) = state.get_focused_window() {
                                if let Some(win) = state.lookup_window(winid.clone()) {
                                    let msg = OutgoingMessage {
                                        recipient: win.owner,
                                        command: KeyDown(KeyDownEvent {
                                            app_id: win.owner,
                                            window_id: win.id,
                                            original_timestamp: 0,
                                            key: kd.key
                                        })
                                    };
                                    if let Err(e) = external_message_sender.send(msg) {
                                        error!("error sending key back to central");
                                    }
                                }
                            }
                        }
                    }
                }
                // APICommand::KeyUp(_) => {}
                APICommand::MouseDown(mme) => {
                    let pt = Point::init(mme.x, mme.y);
                    info!("mouse down at {:?}",pt);
                    if let Some(win) = state.pick_window_at(pt) {
                        debug!("found a window at {:?}", pt);
                        // //if mouse over titlebar, then start a window_move_gesture
                        if win.titlebar_bounds().contains(pt) {
                            debug!("inside the titlebar");
                            gesture = Box::new(WindowDragGesture::init(pt,win.id))
                        }
                        // //if mouse over window_contents, then set window focused
                        if win.content_bounds().contains(pt) {
                            //     //do nothing
                        }
                        state.set_focused_window(win.id);
                    }
                }
                APICommand::MouseMove(evt) => {
                    let bounds = Rect::from_ints(0,0,500,500);
                    let pt = bounds.clamp(&Point::init(evt.x,evt.y));
                    cursor.copy_from(pt);
                    gesture.mouse_move(evt, &mut state);
                }
                APICommand::MouseUp(mme) => {
                    gesture.mouse_up(mme, &mut state);
                    gesture = Box::new(NoOpGesture::init());
                }
                _ => {}
            }
        }
        redraw_screen(&state, &cursor, &cursor_image, &mut plat, &test_pattern);
        plat.service_loop();
        // count += 1;
        // println!("count {}",count);
        // if count > 500 {
            // break;
        // }
    }
    info!("shutting down");
    plat.shutdown();
    info!("waiting for the watch dog");
    watchdog.join().unwrap();
    info!("all done now");
    Ok(())
}

fn capture_screen(state: &WindowManagerState, bounds: Rect, fname: &str) {
    let mut plat = GFXBuffer::new(CD32(), bounds.w as u32, bounds.h as u32, PixelLayout::RGBA());
    plat.clear(&BLACK);
    for win in state.window_list() {
        let (wc,tc) = if state.is_focused_window(win) {
            (FOCUSED_WINDOW_COLOR, FOCUSED_TITLEBAR_COLOR)
        } else {
            (WINDOW_COLOR, TITLEBAR_COLOR)
        };
        plat.draw_rect(win.external_bounds(), &wc, WINDOW_BORDER_WIDTH );
        plat.fill_rect(win.titlebar_bounds(), &tc);
        let bd = win.content_bounds();
        let MAGENTA = ARGBColor::new_rgb(255,0,255);
        plat.fill_rect(bd, &MAGENTA);
        plat.copy_from(win.content_bounds().x, win.content_bounds().y, &win.backbuffer);
    }


    let file = File::create(fname).unwrap();
    let ref mut writ = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writ, plat.width, plat.height); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    // let data = [255, 0, 0, 255, 0, 0, 0, 255]; // An array containing a RGBA sequence. First pixel is red and second pixel is black.
    let data = &plat.data;
    writer.write_image_data(data).unwrap(); // Save

}


fn redraw_screen(state: &WindowManagerState, cursor:&Point, cursor_image:&GFXBuffer, plat: &mut Plat, test_pattern: &GFXBuffer) {            //draw
    let now = Instant::now();
    plat.clear();
    // surf.buf.clear(&BLACK);
    for win in state.window_list() {
        let (wc,tc) = if state.is_focused_window(win) {
            (FOCUSED_WINDOW_COLOR, FOCUSED_TITLEBAR_COLOR)
        } else {
            (WINDOW_COLOR, TITLEBAR_COLOR)
        };
        // surf.buf.draw_rect(win.external_bounds(), wc,WINDOW_BORDER_WIDTH);
        plat.draw_rect(win.external_bounds(), &wc, WINDOW_BORDER_WIDTH );
        plat.fill_rect(win.titlebar_bounds(), &tc);
        let bd = win.content_bounds();
        let MAGENTA = ARGBColor::new_rgb(255,0,255);
        plat.fill_rect(bd, &MAGENTA);
        plat.draw_image(win.content_bounds().x, win.content_bounds().y, &win.backbuffer);
        // surf.copy_from(bd.x, bd.y, &win.backbuffer)
    }
    // plat.fill_rect(Rect::from_ints(cursor.x,cursor.y,10,10), &ARGBColor::new_rgb(255, 255, 255));
    plat.draw_image(cursor.x,cursor.y,cursor_image);
    plat.draw_image(50, 300, test_pattern)
    //info!("drawing {}ms",(now.elapsed().as_millis()));
}

fn make_watchdog(stop: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::spawn({
        move ||{
            let start = Instant::now();
            info!("watchdog thread starting");
            while stop.load(Ordering::Relaxed) == false {
                thread::sleep(Duration::from_millis(1000));
                if start.elapsed().gt(&Duration::from_secs(60)) {
                    info!("timeout of ten seconds. lets bail");
                    stop.store(true, Ordering::Relaxed);
                }
            }
            info!("watchdog thread ending");
        }
    })
}

fn sleep(dur: u64) {
    thread::sleep(Duration::from_millis(dur))
}

fn start_central_server(stop: Arc<AtomicBool>)  {
    info!("running some output");
    thread::spawn(move||{
        let mut child = Command::new("../target/debug/central")
            // .stdin(Stdio::null())
            // .stdout(Stdio::null())
            // .stdout(Stdio::inherit())
            .arg("--debug=true")
            // .env_clear()
            // .env("PATH", "/bin")
            .spawn()
            .expect("child process failed to start")
            ;
        loop {
            sleep(100);
            if stop.load(Ordering::Relaxed) == true {
                info!("killing the child");
                let res = child.kill();
                info!("killed status {:?}",res);
                break;
            }
        }
    });
}

fn start_test_app(stop: Arc<AtomicBool>) {
    info!("starting test app");
    thread::spawn(move||{
        let mut child = Command::new("../target/debug/echo-app")
            .arg("--debug=true")
            .spawn()
            .expect("child process failed to start")
            ;
        loop {
            sleep(100);
            if stop.load(Ordering::Relaxed) == true {
                info!("killing the child");
                let res = child.kill();
                info!("killed status {:?}",res);
                break;
            }
        }
    });
}

#[derive(StructOpt, Debug)]
#[structopt(name = "test-wm", about = "simulates receiving and sending linux-wm events")]
struct Cli {
    #[structopt(long)]
    debug:bool,
    #[structopt(long, default_value="60")]
    timeout:u32,
    #[structopt(long)]
    keyboard:bool,
    #[structopt(long)]
    mouse:bool,
    #[structopt(long)]
    disable_network:bool,
    #[structopt(long)]
    start_server:bool,
    #[structopt(long)]
    start_app1:bool,
}

fn init_setup() -> Cli {
    let args:Cli = Cli::from_args();
    let loglevel = if args.debug { LevelFilter::Debug } else { LevelFilter::Error };

    // create file appender with target file path
    let logfile = FileAppender::builder()
        .build("log/output.log").expect("error setting up file appender");

    // make a config
    let config = Config::builder()
        //add the file appender
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        //now make it
        .build(Root::builder()
            .appender("logfile") // why do we need to mention logfile again?
            .build(loglevel)).expect("error setting up log file");

    log4rs::init_config(config).expect("error initing config");

    thread::sleep(Duration::from_millis(100));
    println!("logging to log/output.log");
    for i in 0..5 {
        info!("        ");
    }
    info!("==============");
    info!("starting new run");
    info!("running with args {:?}",args);
    return args;
}

fn setup_c_handler(stop: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        error!("control C pressed. stopping everything");
        stop.store(true, Ordering::Relaxed)
    }).expect("error setting control C handler");
}
