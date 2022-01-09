use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use env_logger::Env;
use log::{error, info, warn};
use serde::Deserialize;
use uuid::Uuid;
use common::{APICommand, HelloAppResponse, HelloWindowManagerResponse, IncomingMessage, OpenWindowCommand, OpenWindowResponse, Rect};
use structopt::StructOpt;
use common::APICommand::WMConnectResponse;

struct Window {
    id:Uuid,
    bounds:Rect,
}
struct WM {
    id:Uuid,
    stream:TcpStream,
}
struct App {
    id:Uuid,
    stream:TcpStream,
    windows:Vec<Window>,
}
struct CentralState {
    wms:Vec<WM>,
    apps:Vec<App>,
}

impl CentralState {
    fn init() -> CentralState {
        CentralState {
            wms: vec![],
            apps: vec![]
        }
    }
    fn add_app_from_stream(&mut self, stream:TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) {
        let id = Uuid::new_v4();
        self.apps.push(App{ id,stream,windows:vec![] });
        let app = self.apps.iter().find(|a|a.id == id);
        self.spawn_app_handler(id.clone(),app.unwrap().stream.try_clone().unwrap(),sender,stop);
    }
    fn add_window_to_app(&mut self, appid: Uuid, ow: OpenWindowCommand) -> Uuid {
        let winid = Uuid::new_v4();
        let win = Window {
            id: winid,
            bounds: ow.bounds.clone(),
        };
        let mut app = self.apps.iter_mut().find(|a|a.id == appid).unwrap();
        app.windows.push(win);
        return winid
    }
    fn add_wm_from_stream(&mut self, stream:TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) {
        let id = Uuid::new_v4();
        self.wms.push(WM{id,stream});
        let win = self.wms.iter().find(|w|w.id == id).unwrap();
        self.spawn_wm_handler(id.clone(),win.stream.try_clone().unwrap(),sender,stop);
    }
    fn spawn_app_handler(&self, uuid: Uuid, stream: TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut de = serde_json::Deserializer::from_reader(stream);
            loop {
                if stop.load(Ordering::Relaxed) == true {
                    info!("demo-clickgrid thread stopping");
                    break;
                }
                match APICommand::deserialize(&mut de) {
                    Ok(cmd) => {
                        info!("central received command {:?}",cmd);
                        let im = IncomingMessage { source:uuid, command:cmd, };
                        sender.send(im).unwrap();
                    }
                    Err(e) => {
                        error!("error deserializing from demo-clickgrid {:?}",e);
                        stop.store(true,Ordering::Relaxed);
                        break;
                    }
                }
            }
        })
    }
    fn spawn_wm_handler(&self, id: Uuid, stream: TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) -> JoinHandle<()> {
        thread::spawn(move ||{
            let mut de = serde_json::Deserializer::from_reader(stream);
            loop {
                if stop.load(Ordering::Relaxed) == true {
                    info!("wm thread stopping");
                    break;
                }
                match APICommand::deserialize(&mut de) {
                    Ok(cmd) => {
                        info!("central received command {:?}",cmd);
                        let im = IncomingMessage { source:id, command:cmd, };
                        sender.send(im).unwrap();
                    }
                    Err(e) => {
                        error!("error deserializing from window manager {:?}",e);
                        stop.store(true,Ordering::Relaxed);
                        break;
                    }
                }
            }
        })
    }
    fn send_to_app(&mut self, id:Uuid, resp:APICommand) {
        let data = serde_json::to_string(&resp).unwrap();
        let mut app = self.apps.iter_mut().find(|a|a.id == id).unwrap();
        app.stream.write_all(data.as_ref()).expect("failed to send rect");
    }
    fn send_to_wm(&mut self, id:Uuid, resp:APICommand) {
        let data = serde_json::to_string(&resp).unwrap();
        let mut wm = self.wms.iter_mut().find(|a|a.id == id).unwrap();
        wm.stream.write_all(data.as_ref()).expect("failed to send rect");
    }
}

fn main() {
/*
the central server manages system state and routes messages between apps and the window manager

* create network server
* listen for control C to shut things down
* maintain list of apps with list of windows, search for either by ID
* command line opts for the port and network interface
* command line opts for debug
* sets up logging
* when window manager first connects
    * registers as window manager
    * store network connection
    * stores screen size
* when app first connects
    * register app
    * store network connection
    * send message to window manager, if connected
* when app sends draw command
    * route to the window manager
* when window manager sends input event
    * route to the target app
 */

    let args:Cli = init_setup();
    info!("central server starting");
    let state = Arc::new(Mutex::new(CentralState::init()));
    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());
    let (tx, rx) = mpsc::channel::<IncomingMessage>();
    let app_network_thread = start_app_interface(stop.clone(), tx.clone(), state.clone());
    let wm_network_thread = start_wm_interface(stop.clone(), tx.clone(), state.clone());
    // wm_network_thread.join();
    let router_thread = start_router(stop.clone(),rx,state.clone());
    app_network_thread.join();
    info!("central server stopping");
}

fn start_router(stop: Arc<AtomicBool>, rx: Receiver<IncomingMessage>, state: Arc<Mutex<CentralState>>) -> JoinHandle<()> {
    thread::spawn(move||{
        for msg in rx {
            info!("incoming message {:?}",msg);
            match msg.command {
                APICommand::AppConnect(ap) => {
                    info!("app connected {}",msg.source);
                    let resp = APICommand::AppConnectResponse(HelloAppResponse{
                        id: msg.source
                    });
                    state.lock().unwrap().send_to_app(msg.source, resp);
                },
                APICommand::OpenWindowCommand(ow) => {
                    info!("opening window");
                    let winid = state.lock().unwrap().add_window_to_app(msg.source, ow);
                    let resp = APICommand::OpenWindowResponse(OpenWindowResponse{
                        id: winid,
                    });
                    state.lock().unwrap().send_to_app(msg.source, resp);
                },
                APICommand::WMConnect(cmd) => {
                    info!("Window manager connected {}",msg.source);
                    let resp = APICommand::WMConnectResponse(HelloWindowManagerResponse{
                        id:msg.source
                    });
                    state.lock().unwrap().send_to_wm(msg.source, resp)
                }


                _ => {
                    warn!("message not handled {:?}",msg);
                }
            }
        }
    })
}

#[derive(StructOpt, Debug)]
#[structopt(name = "test-wm", about = "simulates receiving and sending linux-wm events")]
struct Cli {
    #[structopt(short, long)]
    debug:bool,
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


fn start_app_interface(stop: Arc<AtomicBool>,
                       tx: Sender<IncomingMessage>,
                       state: Arc<Mutex<CentralState>>
) -> JoinHandle<()> {

    return thread::spawn(move || {
        info!("starting network connection");
        let port = 3333;
        let listener = TcpListener::bind(format!("0.0.0.0:{}",port)).unwrap();
        info!("central listening on port {}",port);
        for stream in listener.incoming() {
            if stop.load(Ordering::Relaxed) { break; }
            match stream {
                Ok(stream) => {
                    info!("got a new app connection");
                    state.lock().unwrap().add_app_from_stream(stream.try_clone().unwrap(),tx.clone(),stop.clone());
                    // let app = App::from_stream(stream.try_clone().unwrap());
                    // handle_client(stream.try_clone().unwrap(),tx.clone(),stop.clone(),state.clone(),app.id);
                    // state.lock().unwrap().add_app(app);
                }
                Err(e) => {
                    error!("error: {}",e);
                }
            }
        }
        drop(listener);
    })
}

fn start_wm_interface(stop:Arc<AtomicBool>,
                      tx: Sender<IncomingMessage>,
                      state: Arc<Mutex<CentralState>>) -> JoinHandle<()> {
    return thread::spawn(move || {
        info!("starting wm connection connection");
        let port = 3334;
        let listener = TcpListener::bind(format!("0.0.0.0:{}",port)).unwrap();
        info!("central listening on port {}",port);
        for stream in listener.incoming() {
            if stop.load(Ordering::Relaxed) { break; }
            match stream {
                Ok(stream) => {
                    info!("got a new wm connection");
                    state.lock().unwrap().add_wm_from_stream(stream.try_clone().unwrap(),tx.clone(),stop.clone());
                    // let wm = WM::init_from_stream(stream.try_clone().unwrap(),state.clone(),tx.clone());
                    // wm.start();
                    // handle_client(stream.try_clone().unwrap(),tx.clone(),stop.clone(),state.clone(),app.id);
                    // state.lock().unwrap().add_app(app);
                }
                Err(e) => {
                    error!("error: {}",e);
                }
            }
        }
        drop(listener);
    })
}
