mod inputtests;

use std::net::{Shutdown, TcpStream};
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::{JoinHandle, Thread};
use std::time::{Duration, Instant};
use std::env;
use std::process::{Child, Command};

use ctrlc;
use env_logger;
use env_logger::Env;
use log::{debug, error, info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use rand::Rng;
use structopt::StructOpt;
use uuid::Uuid;
use common::{APICommand, BLACK, HelloWindowManager, IncomingMessage, Point, Rect};
use common::APICommand::KeyDown;
use common::events::{KeyCode, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
use common::graphics::ColorDepth::CD32;
use common::graphics::GFXBuffer;
use common_wm::{InputGesture, NoOpGesture, OutgoingMessage, start_wm_network_connection, Window, WindowDragGesture, WindowManagerState};

fn main() -> std::io::Result<()>{
    let args:Cli = init_setup();

    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    //try loading a resource
    let cwd = env::current_dir()?;
    info!("cwd is {}", cwd.display());
    let cursor_image:GFXBuffer = GFXBuffer::from_png_file("../resources/cursor.png");

    let mut network_stream:Option<TcpStream> = None;
    //create empty channel first
    let (mut internal_message_sender,
        mut internal_message_receiver) = mpsc::channel::<IncomingMessage>();
    let (mut external_message_sender, rcv2) = mpsc::channel::<OutgoingMessage>();

    start_central_server(stop.clone());
    sleep(1000);

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

    let watchdog = make_watchdog(stop.clone());

    //make thread for fake incoming events. sends to the main event thread
    if args.keyboard {
        inputtests::send_fake_keyboard(stop.clone(), internal_message_sender.clone());
    }
    if args.mouse {
        inputtests::simulate_window_drag(stop.clone(), internal_message_sender.clone());
    }


    //setup the window manager state
    let mut state = WindowManagerState::init();
    //preload a fake app and window
    let fake_app = Uuid::new_v4();
    state.add_app(fake_app);
    let fake_window_uuid = Uuid::new_v4();
    let fake_window_bounds = Rect::from_ints(50,50,200,200);
    state.add_window(fake_app, fake_window_uuid, &fake_window_bounds);

    //start test app
    start_test_app(stop.clone());

    //event processing thread
    start_event_processor(stop.clone(), internal_message_receiver, external_message_sender.clone(), state);
    info!("waiting for the watch dog");
    watchdog.join().unwrap();
    info!("all done now");
    Ok(())
}

fn sleep(dur: u64) {
    thread::sleep(Duration::from_millis(dur))
}

fn start_test_app(stop: Arc<AtomicBool>) {
    let mut child = Command::new("../target/debug/demo-moveplayer")
        .arg("--debug=true")
        .spawn()
        .expect("child process failed to start")
        ;
    thread::spawn(move||{
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

fn start_central_server(stop: Arc<AtomicBool>)  {
    println!("running some output");
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
    thread::spawn(move||{
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


fn start_event_processor(stop: Arc<AtomicBool>, rx: Receiver<IncomingMessage>, tx_out: Sender<OutgoingMessage>, mut state: WindowManagerState) -> JoinHandle<()> {
    return thread::spawn(move || {
        info!("event thread starting");
        //TODO: move the total state to outside the thread, but moves into the thread.

        //TODO:  move the screen to outside this function
        //TODO: move the current gesture holder into the WM state? or just outside here?
        let mut screen = GFXBuffer::new(CD32(),640,480);
        let mut gesture = Box::new(NoOpGesture::init()) as Box<dyn InputGesture>;
        for cmd in rx {
            if stop.load(Ordering::Relaxed) { break; }
            info!("processing event {:?}",cmd);
            match cmd.command {
                APICommand::AppConnectResponse(res) => {
                    info!("adding an app {}",res.app_id);
                    state.add_app(res.app_id);
                },
                APICommand::OpenWindowResponse(ow) => {
                    info!("adding a window to the app");
                    state.add_window(ow.app_id, ow.window_id, &ow.bounds);
                },
                APICommand::DrawRectCommand(dr) => {
                    info!("drawing a rect");
                    if let Some(win) = state.lookup_window(dr.window_id) {
                        win.backbuffer.fill_rect(dr.rect, dr.color);
                    }
                    screen.clear(&BLACK);
                    for win in state.window_list() {
                        screen.copy_from(win.content_bounds().x, win.content_bounds().y, &win.backbuffer)
                    }
                },
                APICommand::KeyDown(kd) => {
                    info!("key down");
                    //now send to a random app's random window, if any
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
                            tx_out.send(msg).unwrap();
                        }
                    }
                },
                APICommand::KeyUp(ku) => {
                    info!("key down")
                },
                APICommand::MouseDown(ev) => {
                    let pt = Point::init(ev.x, ev.y);
                    info!("mouse down at {:?}",pt);
                    state.dump();
                    //if inside a window
                    if let Some(win) = state.pick_window_at(pt.clone()) {
                        debug!("found a window at {:?}", pt);
                        let id = win.id.clone();
                        // //if mouse over titlebar, then start a window_move_gesture
                        if win.titlebar_bounds().contains(pt) {
                            gesture = Box::new(WindowDragGesture::init(pt,id));
                        }
                        // //if mouse over window_contents, then set window focused
                        if win.content_bounds().contains(pt) {
                        //     //do nothing
                        }
                        state.set_focused_window(id);
                    }
                    gesture.mouse_down(ev, &mut state);
                }
                APICommand::MouseMove(ev) => {
                    info!("mouse move");
                    gesture.mouse_move(ev, &mut state);
                },
                APICommand::MouseUp(ev) => {
                    info!("mouse up");
                    gesture.mouse_up(ev, &mut state);
                    gesture = Box::new(NoOpGesture::init());
                    state.dump();
                },
                _ => {}
            };
        }
        info!("event thread ending");
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
