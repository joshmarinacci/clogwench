use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use ctrlc;
use env_logger;
use env_logger::Env;
use log::{error, info};
use serde::Deserialize;
use serde_json;
use structopt::StructOpt;
use uuid::Uuid;

use common::{APICommand, App, ARGBColor, CentralState, IncomingMessage, Rect, Window, Point};
use common::events::{KeyCode, KeyDownEvent, MouseDownEvent};
use common::graphics::{Screen, Surface};

struct HeadlessScreen {
    bounds:Rect,
}
impl HeadlessScreen {
    fn init(rect:Rect) -> HeadlessScreen {
        HeadlessScreen {
            bounds: rect,
        }
    }
}
impl Screen for HeadlessScreen {
    fn get_bounds(&self) -> &Rect {
        return &self.bounds;
    }
}

struct HeadlessSurface {
    bounds:Rect
}
impl HeadlessSurface {
    pub fn init(rect:Rect) -> HeadlessSurface {
        HeadlessSurface {
            bounds:rect,
        }
    }
}
impl Surface for HeadlessSurface {
    fn get_bounds(&self) -> &Rect {
        return &self.bounds
    }
    fn fill_rect(&self, rect: &Rect, color: &ARGBColor) {
    }
}

struct ComStream {
}

struct WindowSurface {
    window:Window,
    surface:HeadlessSurface,
    comm:CommStream,
}

struct WindowManager {
    focused_window:Option<WindowSurface>,
    screen:HeadlessScreen,
    surface_order:Vec<WindowSurface>,
    last_position:Point,
}

impl WindowManager {
    pub fn init(screen:HeadlessScreen) -> WindowManager {
        WindowManager {
            focused_window: None,
            screen,
            surface_order: Default::default(),
            last_position:Point::init(30, 30),
        }
    }

    // position when automatically when created
    pub fn add_window(&mut self, win: Window) {
        let surf = HeadlessSurface::init(win.bounds.clone());
        let mut ws = WindowSurface {
            window: win,
            surface: surf,
            comm: ()
        };
        self.last_position = self.last_position.add(Point::make(100, 0));
        ws.window.bounds.set_position(&self.last_position);
        self.surface_order.push(ws);
    }

    // dispatch keyboard event to the focused window, if any
    pub fn dispatch_keyboard(&self, ev:KeyDownEvent) {
        if let Some(win) = &self.focused_window {
            win.comm.dispatch(ev)
        }
    }

    // redraw all windows to the screen
    pub fn redraw_screen(&self) {
        for ws in self.surface_order {
            self.screen.composite(ws.surface);
            info!("drawing a surface {:?}",ws.surface.bounds);
        }
    }

    // find window that the mouse is in
    pub fn mouse_down(&mut self, ev: MouseDownEvent) {
        // if window found, then set it as focused
        for ws in self.surface_order {
            if ws.bounds.contains(ev.position) {
                // raise window to the top
                self.focused_window = Some(ws);
                self.raise_window_to_top(ws);
                // trigger a refresh
                self.redraw_screen()
            }
        }
    }

    fn raise_window_to_top(&mut self, ws:WindowSurface) {
        let n = self.surface_order.index_of(ws);
        self.surface_order.remove(n);
        self.surface_order.push(ws);
    }
}

fn main() {
    let args:Cli = init_setup();

    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    let (tx, rx) = mpsc::channel::<IncomingMessage>();

    let state:Arc<Mutex<CentralState>> = Arc::new(Mutex::new(CentralState::init()));

    let screen = start_headless_screen(Rect::from_ints(0, 0, 640, 480));
    start_network_server(stop.clone(), tx, state.clone());
    start_event_processor(stop.clone(), rx, state.clone());

    if args.keyboard {
        send_fake_keyboard(state.clone(), stop.clone());
    }

    let timeout_handle = start_timeout(stop.clone(), args.timeout);
    timeout_handle.join().unwrap();
    info!("all done now");
}

fn start_headless_screen(rect: Rect) -> HeadlessScreen {
    return HeadlessScreen::init(rect)
}


fn start_event_processor(stop: Arc<AtomicBool>,
                         rx: Receiver<IncomingMessage>,
                         state: Arc<Mutex<CentralState>>
) -> JoinHandle<()> {
    return thread::spawn(move || {
        let mut wm = WindowManager::init();
        for cmd in rx {
            if stop.load(Ordering::Relaxed) { break; }
            info!("processing event {:?}",cmd);
            match cmd.command {
                APICommand::OpenWindowCommand(ow) => {
                    info!("adding a window to the app");
                    let win = Window::from_rect(ow.bounds);
                    wm.add_window(&win);
                    state.lock().unwrap().add_window(cmd.appid,win);
                    wm.refresh();
                },
                APICommand::DrawRectCommand(dr) => {
                    info!("drawing a rect");
                    if let Some(app) = state.lock().unwrap().find_app_by_id(cmd.appid) {
                        wm.lookup_surface_for_window(dr.window)
                            .unwrap().fill_rect(&dr.rect,&dr.color);
                        wm.refresh();
                    }
                },
                APICommand::KeyDown(kd) => {
                    info!("key down")
                },
                APICommand::KeyUp(ku) => {
                    info!("key down")
                },
                APICommand::MouseDown(ku) => {
                    info!("mouse down");
                    wm.mouse_down(ku);
                },
                APICommand::MouseMove(ku) => {
                    info!("mouse move")
                },
                APICommand::MouseUp(ku) => {
                    info!("mouse up")
                },
            };
        }
    });
}

fn start_network_server(stop: Arc<AtomicBool>,
                        tx: Sender<IncomingMessage>,
                        state: Arc<Mutex<CentralState>>) -> JoinHandle<()> {

    return thread::spawn(move || {
        info!("starting network connection");
        let port = 3333;
        let listener = TcpListener::bind(format!("0.0.0.0:{}",port)).unwrap();
        info!("linux-wm listening on port {}",port);
        for stream in listener.incoming() {
            if stop.load(Ordering::Relaxed) { break; }
            match stream {
                Ok(stream) => {
                    info!("got a new connection");
                    let app = App::from_stream(stream.try_clone().unwrap());
                    handle_client(stream.try_clone().unwrap(),tx.clone(),stop.clone(),state.clone(),app.id);
                    state.lock().unwrap().add_app(app);
                }
                Err(e) => {
                    error!("error: {}",e);
                }
            }
        }
        drop(listener);
    })
}


fn send_fake_keyboard(state: Arc<Mutex<CentralState>>, stop: Arc<AtomicBool>) {
    thread::spawn({
        move || {
            loop {
                if stop.load(Ordering::Relaxed) { break; }
                let cmd: APICommand = APICommand::KeyDown(KeyDownEvent {
                    original_timestamp: 0,
                    key: KeyCode::ARROW_RIGHT
                });
                let data = serde_json::to_string(&cmd).unwrap();
                info!("sending fake event {:?}",cmd);
                {
                    for (uuid, mut app) in state.lock().unwrap().app_list() {
                        app.connection.write_all(data.as_ref()).expect("failed to send rect");
                    }
                }
                thread::sleep(Duration::from_millis(1000));
            }
        }
    });

}

fn handle_client(stream: TcpStream, tx: Sender<IncomingMessage>, stop: Arc<AtomicBool>, _state: Arc<Mutex<CentralState>>, appid: Uuid) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut de = serde_json::Deserializer::from_reader(stream);
        loop {
            if stop.load(Ordering::Relaxed) == true {
                info!("demo-clickgrid thread stopping");
                break;
            }
            match APICommand::deserialize(&mut de) {
                Ok(cmd) => {
                    //info!("linux-wm received command {:?}",cmd);
                    let im = IncomingMessage { appid, command:cmd, };
                    tx.send(im).unwrap();
                }
                Err(e) => {
                    error!("error deserializing from demo-clickgrid {:?}",e);
                    break;
                }
            }
        }
    })
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
    #[structopt(short, long)]
    keyboard:bool,
}

fn init_setup() -> Cli {
    let args:Cli = Cli::from_args();
    let loglevel = if args.debug { "debug"} else { "error"};
    env_logger::Builder::from_env(Env::default().default_filter_or(loglevel)).init();
    info!("running with args {:?}",args);
    return args;
}

fn setup_c_handler(stop: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        error!("control C pressed. stopping everything");
        stop.store(true, Ordering::Relaxed)
    }).expect("error setting control C handler");
}
