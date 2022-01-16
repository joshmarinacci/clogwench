use structopt::StructOpt;
use std::{env, thread};
use std::net::TcpStream;
use std::process::Command;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log::{error, info, LevelFilter};
use uuid::Uuid;
use common::graphics::GFXBuffer;
use common::{APICommand, ARGBColor, IncomingMessage, Point, Rect, WHITE};
use common_wm::{FOCUSED_TITLEBAR_COLOR, FOCUSED_WINDOW_COLOR, OutgoingMessage, start_wm_network_connection, TITLEBAR_COLOR, WINDOW_BORDER_WIDTH, WINDOW_COLOR, WindowManagerState};
use plat::{make_plat, Plat};

fn main() -> std::io::Result<()>{
    //initial setup
    let args:Cli = init_setup();
    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    let watchdog = make_watchdog(stop.clone());

    //load the cursor image
    let cwd = env::current_dir()?;
    info!("cwd is {}", cwd.display());
    let cursor_image:GFXBuffer = GFXBuffer::from_png_file("../resources/cursor.png");

    let mut network_stream:Option<TcpStream> = None;
    //create empty channel first
    let (mut internal_message_sender,
        mut internal_message_receiver) = mpsc::channel::<IncomingMessage>();
    let (mut external_message_sender, rcv2) = mpsc::channel::<OutgoingMessage>();

    //start the central server
    // start_central_server(stop.clone());
    sleep(1000);

    //connect to the central server
    if !args.disable_network {
        info!("connecting to the central server");
        //open network connection
        let conn = start_wm_network_connection(stop.clone())
            .expect("error connecting to the central server");
        conn.send_hello();
        network_stream = Option::from(conn.stream);
        internal_message_sender = conn.tx_in;
        external_message_sender = conn.tx_out;
        info!("fully connected to the network now");
    } else {
        info!("skipping the network connection");
    }


    //make thread for fake incoming events. sends to the main event thread
    // if args.keyboard {
    //     inputtests::send_fake_keyboard(stop.clone(), internal_message_sender.clone());
    // }
    // if args.mouse {
    //     inputtests::simulate_window_drag(stop.clone(), internal_message_sender.clone());
    // }


    //setup the window manager state
    let mut state = WindowManagerState::init();
    //preload a fake app and window
    let fake_app = Uuid::new_v4();
    state.add_app(fake_app);
    let fake_window_uuid = Uuid::new_v4();
    let fake_window_bounds = Rect::from_ints(50,50,200,200);
    state.add_window(fake_app, fake_window_uuid, &fake_window_bounds);

    //start test app
    // start_test_app(stop.clone());


    //make the platform specific graphics
    let mut plat = make_plat(stop.clone(), internal_message_sender.clone()).unwrap();
    println!("Made a plat");
    plat.register_image2();

    let bounds:Rect = plat.get_screen_bounds();
    println!("screen bounds are {:?}",bounds);
    let mut cursor:Point = Point::init(0,0);

    let mut count = 0;
    loop {
        if stop.load(Ordering::Relaxed)==true { break; }
        plat.service_input();
        //info!("checking for incoming events");
        for cmd in internal_message_receiver.try_iter() {
            info!("incoming {:?}",cmd);
            if stop.load(Ordering::Relaxed) == true { break; }
            match cmd.command {
                // APICommand::AppConnect(_) => {}
                // APICommand::AppConnectResponse(_) => {}
                // APICommand::WMConnect(_) => {}
                // APICommand::WMConnectResponse(_) => {}
                // APICommand::OpenWindowCommand(_) => {}
                // APICommand::OpenWindowResponse(_) => {}
                // APICommand::DrawRectCommand(_) => {}
                // APICommand::KeyDown(_) => {}
                // APICommand::KeyUp(_) => {}
                // APICommand::MouseDown(_) => {}
                APICommand::MouseMove(evt) => {
                    cursor.x = evt.x;
                    cursor.y = evt.y;

                }
                // APICommand::MouseUp(_) => {}
                _ => {}
            }
        }
        redraw_screen(&state, &cursor, &cursor_image, &mut plat);
        plat.service_loop();
        count += 1;
        // println!("count {}",count);
        if count > 500 {
            break;
        }
    }
    info!("shutting down");
    plat.shutdown();
    info!("waiting for the watch dog");
    watchdog.join().unwrap();
    info!("all done now");
    Ok(())
}


fn redraw_screen(state: &WindowManagerState, cursor:&Point, cursor_image:&GFXBuffer, plat: &mut Plat) {            //draw
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
        plat.fill_rect(bd, &WHITE);
        // surf.copy_from(bd.x, bd.y, &win.backbuffer)
    }
    plat.fill_rect(Rect::from_ints(cursor.x,cursor.y,10,10), &ARGBColor::new_rgb(255, 255, 225));
    plat.draw_image(cursor.x,cursor.y,cursor_image);
    // surf.copy_from(cursor.x, cursor.y, &cursor_image);
    // surf.sync();
    //info!("drawing {}ms",(now.elapsed().as_millis()));
}

fn make_watchdog(stop: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::spawn({
        move ||{
            let start = Instant::now();
            info!("watchdog thread starting");
            while stop.load(Ordering::Relaxed) == false {
                thread::sleep(Duration::from_millis(1000));
                if start.elapsed().gt(&Duration::from_secs(10)) {
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
        let mut child = Command::new("../target/debug/demo-moveplayer")
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
    disable_network:bool
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
