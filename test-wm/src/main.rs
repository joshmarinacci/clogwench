use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::env;

use ctrlc;
use env_logger;
use env_logger::Env;
use log::{debug, info, LevelFilter};
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
use common_wm::{OutgoingMessage, start_wm_network_connection, Window, WindowManagerState};

fn main() -> std::io::Result<()>{
    let args:Cli = init_setup();

    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    //try loading a resource
    let cwd = env::current_dir()?;
    println!("cwd is {}", cwd.display());
    let cursor_image:GFXBuffer = GFXBuffer::from_png_file("../resources/cursor.png");

    //open network connection
    let conn = start_wm_network_connection(stop.clone())
        .expect("error connecting to the central server");

    //send hello window manager
    let msg = OutgoingMessage {
        recipient: Default::default(),
        command: APICommand::WMConnect(HelloWindowManager {
        })
    };
    conn.tx_out.send(msg).unwrap();

    let resp = conn.rx_in.recv().unwrap();
    let selfid = if let APICommand::WMConnectResponse(res) = resp.command {
        info!("got response back from the server {:?}",res);
        res.wm_id
    } else {
        panic!("did not get the window manager connect response. gah!");
    };

    let watchdog = make_watchdog(stop.clone(),conn.stream.try_clone().unwrap());

    //make thread for fake incoming events. sends to the main event thread
    if args.keyboard {
        send_fake_keyboard(stop.clone(), conn.tx_in.clone());
    }
    if args.mouse {
        send_fake_mouse(stop.clone(), conn.tx_in.clone());
    }

    //event processing thread
    start_event_processor(stop.clone(), conn.rx_in, conn.tx_out.clone());
        //draw commands. can immediately draw to the fake screen
        //app added, add to own app list
        //window added, add to own app window list
        //key pressed in event thread
        //on keypress, send to app owner of focused window
        //on mouse press, maybe change the focused window
        //on mouse press, send to window under the cursor
        //can all state live on this thread?
    info!("waiting for the watch dog");
    watchdog.join().unwrap();
    info!("all done now");
    Ok(())
}

fn send_fake_mouse(stop: Arc<AtomicBool>, sender: Sender<IncomingMessage>) {
    thread::spawn({
        info!("starting fake mouse events");
        move || {
            let mut rng = rand::thread_rng();
            loop {
                if stop.load(Ordering::Relaxed) { break; }
                //mouse down at 55,55, drag to 200,100, release
                let command: APICommand = APICommand::MouseDown(MouseDownEvent {
                    original_timestamp: 0,
                    button: MouseButton::Primary,
                    x: 55,//rng.gen_range(0..500),
                    y: 55,
                });
                sender.send(IncomingMessage{
                    source: Default::default(),
                    command
                }).unwrap();
                thread::sleep(Duration::from_millis(1000));

                //drag over 5 spots to the right
                for off in 0..5 {
                    sender.send(IncomingMessage{
                        source: Default::default(),
                        command:APICommand::MouseMove(MouseMoveEvent{
                            original_timestamp: 0,
                            button: MouseButton::Primary,
                            x: 55+off*10,
                            y: 55
                        })
                    }).unwrap();
                    thread::sleep(Duration::from_millis(1000))
                }

                //release
                let command: APICommand = APICommand::MouseUp(MouseUpEvent {
                    original_timestamp: 0,
                    button: MouseButton::Primary,
                    x: 55+4*10,//rng.gen_range(0..500),
                    y: 55,
                });
                sender.send(IncomingMessage{ source: Default::default(),  command }).unwrap();
                thread::sleep(Duration::from_millis(1000));
                break;
            }
        }
    });

}

fn make_watchdog(stop: Arc<AtomicBool>, stream: TcpStream) -> JoinHandle<()> {
    thread::spawn({
        move ||{
            info!("watchdog thread starting");
            loop {
                if stop.load(Ordering::Relaxed) {
                    info!("shutting down the network");
                    stream.shutdown(Shutdown::Both).unwrap();
                    break;
                }
                thread::sleep(Duration::from_millis(1000))
            }
            info!("watchdog thread ending");
        }
    })
}

trait InputGesture {
    fn mouse_down(&mut self, evt:MouseDownEvent, state:&mut WindowManagerState);
    fn mouse_move(&mut self, evt:MouseMoveEvent, state:&mut WindowManagerState);
    fn mouse_up(  &mut self, evt:MouseUpEvent, state:&mut WindowManagerState);
}


struct NoOpGesture {

}

impl NoOpGesture {
    fn init() -> NoOpGesture {
        NoOpGesture {}
    }
}

impl InputGesture for NoOpGesture {
    fn mouse_down(&mut self, evt: MouseDownEvent, state:&mut WindowManagerState) {
        info!("got a mouse down event {:?}",evt);
    }

    fn mouse_move(&mut self, evt: MouseMoveEvent, state:&mut WindowManagerState) {
        todo!()
    }

    fn mouse_up(&mut self, evt: MouseUpEvent, state:&mut WindowManagerState) {
        todo!()
    }
}
struct WindowDragGesture {
    start:Point,
    winid:Uuid,
}
impl WindowDragGesture {
    fn init(start: Point, win: Uuid) -> WindowDragGesture {
        WindowDragGesture {
            start:Point::init(0,0),
            winid:win
        }
    }
}
impl InputGesture for WindowDragGesture {
    fn mouse_down(&mut self, evt: MouseDownEvent, state:&mut WindowManagerState) {
        info!("WDG: mouse down {:?}",evt);
        self.start = Point::init(evt.x,evt.y);
    }

    fn mouse_move(&mut self, evt: MouseMoveEvent, state:&mut WindowManagerState) {
        info!("WDG: mouse move {:?}",evt);
        let curr = Point::init(evt.x,evt.y);
        let diff = curr.subtract(self.start);
        info!("dragging window {} by {:?}",self.winid,diff)
    }

    fn mouse_up(&mut self, evt: MouseUpEvent, state:&mut WindowManagerState) {
        info!("WDG completed");
        let curr = Point::init(evt.x,evt.y);
        info!("new window position is {} to {:?}",self.winid,curr);
        if let Some(win) = state.lookup_window(self.winid) {
            win.position.x = curr.x;
            win.position.y = curr.y;
        }
    }
}

fn start_event_processor(stop: Arc<AtomicBool>, rx: Receiver<IncomingMessage>, tx_out: Sender<OutgoingMessage>) -> JoinHandle<()> {
    return thread::spawn(move || {
        info!("event thread starting");
        let mut state = WindowManagerState::init();
        let fake_app = Uuid::new_v4();
        state.add_app(fake_app);
        let fake_window_uuid = Uuid::new_v4();
        let fake_window_bounds = Rect::from_ints(50,50,200,200);
        state.add_window(fake_app, fake_window_uuid, &fake_window_bounds);

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

fn send_fake_keyboard(stop: Arc<AtomicBool>, sender: Sender<IncomingMessage>) {
    thread::spawn({
        move || {
            loop {
                if stop.load(Ordering::Relaxed) { break; }
                let command: APICommand = APICommand::KeyDown(KeyDownEvent {
                    app_id: Default::default(),
                    window_id: Default::default(),
                    original_timestamp: 0,
                    key: KeyCode::ARROW_RIGHT
                });
                sender.send(IncomingMessage{
                    source: Default::default(),
                    command
                }).unwrap();
                thread::sleep(Duration::from_millis(1000));
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

fn setup_c_handler(stop: Arc<AtomicBool>) {
    // ctrlc::set_handler(move || {
    //     error!("control C pressed. stopping everything");
    //     stop.store(true, Ordering::Relaxed)
    // }).expect("error setting control C handler");
}
