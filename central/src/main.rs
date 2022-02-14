use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::{io, thread};
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use env_logger::Env;
use log::{error, info, LevelFilter, warn};
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use serde::Deserialize;
use uuid::Uuid;
use common::{APICommand, APP_MANAGER_PORT, DEBUG_PORT, DebugMessage, HelloAppResponse, HelloWindowManagerResponse, IncomingMessage, OpenWindowCommand, OpenWindowResponse, Rect, WINDOW_MANAGER_PORT};
use structopt::StructOpt;

struct Window {
    id:Uuid,
    bounds:Rect,
}
struct WM {
    id:Uuid,
    stream:TcpStream,
}
struct Debugger {
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
    debuggers:Vec<Debugger>,
}

impl CentralState {
    fn init() -> CentralState {
        CentralState {
            wms: vec![],
            apps: vec![],
            debuggers: vec![]
        }
    }
    fn add_app_from_stream(&mut self, stream:TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) {
        let id = Uuid::new_v4();
        self.apps.push(App{ id,stream,windows:vec![] });
        if let Some(app) = self.apps.iter().find(|a|a.id == id) {
            self.spawn_app_handler(id.clone(), app.stream.try_clone().unwrap(), sender, stop);
        }
    }
    fn add_window_to_app(&mut self, appid: Uuid, ow: &OpenWindowCommand) -> Uuid {
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
        if let Some(wm) = self.wms.iter().find(|w|w.id == id) {
            self.spawn_wm_handler(id.clone(), wm.stream.try_clone().unwrap(), sender, stop);
        }
    }
    fn add_debugger_from_stream(&mut self, stream: TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) {
        let id = Uuid::new_v4();
        self.debuggers.push(Debugger{id,stream});
        if let Some(c) = self.debuggers.iter().find(|w|w.id == id) {
            self.spawn_debugger_handler(id.clone(), c.stream.try_clone().unwrap(), sender, stop);
        }
    }

    fn spawn_app_handler(&self, appid: Uuid, stream: TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) -> JoinHandle<()> {
        thread::spawn(move || {
            info!("app thread starting: {}",appid);
            println!("CENTRAL: app {} thread starting",appid);
            stream.set_nonblocking(false).unwrap();
            let stream2 = stream.try_clone().unwrap();
            let mut de = serde_json::Deserializer::from_reader(stream);
            loop {
                if stop.load(Ordering::Relaxed) == true {
                    println!("CENTRAL: app thread stopping");
                    break;
                }
                match APICommand::deserialize(&mut de) {
                    Ok(cmd) => {
                        info!("central received command {:?}",cmd);
                        let im = IncomingMessage { source: appid, command:cmd, };
                        if let Err(e) = sender.send(im) {
                            error!("error sending command {}",e);
                        }
                    }
                    Err(e) => {
                        error!("error deserializing from demo-clickgrid {:?}",e);
                        stream2.shutdown(Shutdown::Both);
                        break;
                    }
                }
            }
            // info!("app thread ending {}",appid);
            println!("CENTRAL: app {} thread ending",appid);
        })
    }
    fn spawn_wm_handler(&self, wm_id: Uuid, stream: TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) -> JoinHandle<()> {
        thread::spawn(move ||{
            println!("CENTRAL: WM thread starting: {}",wm_id);
            stream.set_nonblocking(false).unwrap();
            // info!("wm thread starting: {}",wm_id);
            let stream2 = stream.try_clone().unwrap();
            let mut de = serde_json::Deserializer::from_reader(stream);
            loop {
                if stop.load(Ordering::Relaxed) == true {
                    println!("CENTRAL: wm thread stopping");
                    stream2.shutdown(Shutdown::Both);
                    break;
                }
                match IncomingMessage::deserialize(&mut de) {
                    Ok(cmd) => {
                        info!("central received wm command {:?}",cmd);
                        sender.send(IncomingMessage{ source: wm_id, command: cmd.command }).unwrap();
                    }
                    Err(e) => {
                        error!("error deserializing from window manager {:?}",e);
                        stream2.shutdown(Shutdown::Both);
                        break;
                    }
                }
            }
            println!("CENTRAL: WM {} thread ending", wm_id);
        })
    }
    fn spawn_debugger_handler(&self, id:Uuid, stream: TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) -> JoinHandle<()>{
        thread::spawn(move ||{
            println!("CENTRAL: debugger thread starting: {}",id);
            stream.set_nonblocking(false).unwrap();
            let stream2 = stream.try_clone().unwrap();
            let mut de = serde_json::Deserializer::from_reader(stream);
            loop {
                if stop.load(Ordering::Relaxed) == true {
                    println!("CENTRAL: debugger thread stopping");
                    stream2.shutdown(Shutdown::Both);
                    break;
                }
                match DebugMessage::deserialize(&mut de) {
                    Ok(cmd) => {
                        println!("CENTRAL: received debugger command {:?}",cmd);
                        sender.send(IncomingMessage{source:id, command:APICommand::Debug(cmd)}).unwrap();
                    }
                    Err(e) => {
                        // if e.kind() == io::ErrorKind::WouldBlock {
                            // println!("need to loop again, {}",name);
                        // } else {
                        println!("CENTRAL: error deserializing from debugger {:?}",e);
                        stream2.shutdown(Shutdown::Both);
                        break;
                    }
                }
            }
            println!("CENTRAL: debugger thread ending: {}",id);
        })
    }

    fn send_to_app(&mut self, id:Uuid, resp: APICommand) {
        println!("CENTRAL: sending to app {:?}",resp);
        let data = serde_json::to_string(&resp).unwrap();
        if let Some(app) = self.apps.iter_mut().find(|a|a.id == id){
            app.stream.write_all(data.as_ref()).expect("failed to send rect");
        }
    }
    fn send_to_all_apps(&mut self, resp: APICommand) {
        println!("CENTRAL: sending to all apps {:?}",resp);
        let im = IncomingMessage {
            source: Default::default(),
            command: resp,
        };
        let data = serde_json::to_string(&im).unwrap();
        for app in self.apps.iter_mut() {
            app.stream.write_all(data.as_ref()).expect("failed to send to all wm")
        }
    }
    fn send_to_wm(&mut self, id:Uuid, resp: APICommand) {
        println!("CENTRAL: sending to wm {:?}",resp);
        let im = IncomingMessage {
            source: Default::default(),
            command: resp,
        };
        let data = serde_json::to_string(&im).unwrap();
        let mut wm = self.wms.iter_mut().find(|a|a.id == id).unwrap();
        wm.stream.write_all(data.as_ref()).expect("failed to send data to wm");
    }
    fn send_to_all_wm(&mut self, resp: APICommand) {
        println!("CENTRAL: sending to all wm {:?}",resp);
        let im = IncomingMessage {
            source: Default::default(),
            command: resp,
        };
        let data = serde_json::to_string(&im).unwrap();
        for wm in self.wms.iter_mut() {
            wm.stream.write_all(data.as_ref()).expect("failed to send to all wm")
        }
    }
    fn send_to_debugger(&mut self, resp: DebugMessage) {
        println!("CENTRAL: sending to debugger {:?}",resp);
        let data = serde_json::to_string(&resp).unwrap();
        for dbg in self.debuggers.iter_mut() {
            dbg.stream.write_all(data.as_ref()).expect("CENTRAL: error sending to debugger");
        }

    }
}

fn main() {
    let args:Cli = init_setup();
    info!("central server starting");
    println!("central server starting");
    let state = Arc::new(Mutex::new(CentralState::init()));
    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());
    let (tx, rx) = mpsc::channel::<IncomingMessage>();
    // let cls = move |stream:TcpStream, tx:Sender<IncomingMessage>, stop:Arc<AtomicBool>, state:Arc<Mutex<CentralState>>| {
    // };
    let app_network_thread = start_network_interface(stop.clone(),tx.clone(), state.clone(),
                                                     String::from("app"),APP_MANAGER_PORT,
                                                     |stream,tx,stop,state|{
                                                         state.lock().unwrap().add_app_from_stream(stream,tx,stop.clone());
                                                     });
    let wm_network_thread = start_network_interface(stop.clone(),tx.clone(), state.clone(),
                                                    String::from("winman"),
                                                    WINDOW_MANAGER_PORT,
                                                    |stream,tx,stop,state|{
                                                        state.lock().unwrap().add_wm_from_stream(stream.try_clone().unwrap(),tx.clone(),stop.clone());
                                                    });
    let debug_network_thread = start_network_interface(stop.clone(),tx.clone(), state.clone(),
                                                    String::from("debug"),
                                                    DEBUG_PORT,
                                                    |stream,tx,stop,state|{
                                                        state.lock().unwrap().add_debugger_from_stream(stream.try_clone().unwrap(), tx.clone(), stop.clone());
                                                    });
    let router_thread = start_router(stop.clone(),rx,state.clone());
    info!("waiting for the app network thread to end");
    println!("CENTRAL: waiting for the app thread to end");
    app_network_thread.join();
    wm_network_thread.join();
    debug_network_thread.join();
    info!("central server stopping");
}

fn start_router(stop: Arc<AtomicBool>, rx: Receiver<IncomingMessage>, state: Arc<Mutex<CentralState>>) -> JoinHandle<()> {
    thread::spawn(move||{
        println!("CENTRAL: router thread starting");
        for msg in rx {
            println!("CENTRAL: incoming message {:?}",msg);
            match msg.command {
                APICommand::Debug(DebugMessage::HelloDebugger) => {
                    let resp = DebugMessage::HelloDebuggerResponse;
                    state.lock().unwrap().send_to_debugger(resp);
                }
                APICommand::Debug(DebugMessage::FakeMouseEvent(evt)) => {
                    state.lock().unwrap().send_to_all_wm(APICommand::MouseDown(evt));
                }
                APICommand::Debug(DebugMessage::BackgroundReceivedMouseEvent) => {
                    state.lock().unwrap().send_to_debugger(DebugMessage::BackgroundReceivedMouseEvent);
                }
                APICommand::Debug(DebugMessage::WindowFocusChanged(str)) => {
                    state.lock().unwrap().send_to_debugger(DebugMessage::WindowFocusChanged(str));
                }
                APICommand::Debug(DebugMessage::AppLog(str)) => {
                    state.lock().unwrap().send_to_debugger(DebugMessage::AppLog(str));
                }
                APICommand::Debug(DebugMessage::ScreenCapture(rect, str)) => {
                    state.lock().unwrap().send_to_all_wm(APICommand::Debug(DebugMessage::ScreenCapture(rect, str)));
                }
                APICommand::Debug(DebugMessage::ScreenCaptureResponse()) => {
                    state.lock().unwrap().send_to_debugger(DebugMessage::ScreenCaptureResponse());
                }
                APICommand::Debug(DebugMessage::RequestServerShutdown) => {
                    {
                        let mut st = state.lock().unwrap();
                        println!("CENTRAL sending out shutdown messages and waiting a second");
                        st.send_to_all_wm(APICommand::SystemShutdown);
                        st.send_to_all_apps(APICommand::SystemShutdown);
                    }
                    thread::sleep(Duration::from_millis(1000));
                    println!("CENTRAL sending stop command to all threads");
                    stop.store(true, Ordering::Relaxed);
                    thread::sleep(Duration::from_millis(1000));
                    println!("CENTRAL the server is really ending");
                    break;
                }
                APICommand::AppConnect(ap) => {
                    info!("app connected {}",msg.source);
                    let resp = APICommand::AppConnectResponse(HelloAppResponse{
                        app_id: msg.source
                    });
                    state.lock().unwrap().send_to_app(msg.source, resp.clone());
                    state.lock().unwrap().send_to_all_wm(resp.clone());
                    state.lock().unwrap().send_to_debugger(DebugMessage::AppConnected(String::from("foo")))
                },
                APICommand::OpenWindowCommand(ow) => {
                    // info!("opening window");
                    let winid = state.lock().unwrap().add_window_to_app(msg.source, &ow);
                    let resp = APICommand::OpenWindowResponse(OpenWindowResponse{
                        app_id: msg.source,
                        window_id: winid,
                        window_type: ow.window_type.clone(),
                        bounds: ow.bounds.clone(),
                    });
                    state.lock().unwrap().send_to_app(msg.source, resp.clone());
                    state.lock().unwrap().send_to_all_wm(resp.clone());
                    state.lock().unwrap().send_to_debugger(DebugMessage::WindowOpened(String::from("foo")));
                },
                APICommand::WMConnect(cmd) => {
                    let resp = APICommand::WMConnectResponse(HelloWindowManagerResponse{
                        wm_id:msg.source
                    });
                    state.lock().unwrap().send_to_wm(msg.source, resp.clone());
                    state.lock().unwrap().send_to_debugger(DebugMessage::WindowManagerConnected);
                }
                APICommand::DrawRectCommand(cmd) => {
                    state.lock().unwrap().send_to_all_wm(APICommand::DrawRectCommand(cmd));
                },
                APICommand::KeyDown(e) => {
                    state.lock().unwrap().send_to_app(e.app_id, APICommand::KeyDown(e))
                }
                APICommand::MouseDown(e) => {
                    state.lock().unwrap().send_to_app(e.app_id, APICommand::MouseDown(e))
                }
                _ => {
                    println!("CENTRAL: message not handled {:?}",msg);
                }
            }
        }
        println!("CENTRAL: router thread quitting");
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
    // let loglevel = if args.debug { LevelFilter::Debug } else { LevelFilter::Error };
    let loglevel = LevelFilter::Debug;
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
    println!("CENTRAL is logging to log/output.log");
    for i in 0..5 {
        info!("        ");
    }
    info!("==============");
    info!("starting new run");
    info!("running with args {:?}",args);
    println!("debug level is {:?}", loglevel);
    return args;
}

fn setup_c_handler(stop: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        println!("control C pressed. stopping everything");
        stop.store(true, Ordering::Relaxed)
    }).expect("error setting control C handler");
}

fn start_network_interface<F>(stop: Arc<AtomicBool>,
                           tx: Sender<IncomingMessage>,
                           state: Arc<Mutex<CentralState>>,
                           name: String,
                           port: i32,
                           cb: F
) -> JoinHandle<()>
    where
        F: Fn(TcpStream, Sender<IncomingMessage>, Arc<AtomicBool>, Arc<Mutex<CentralState>>),
        F: Send + 'static,
{

    return thread::spawn(move || {
        println!("CENTRAL: starting {} network connection",name);
        let listener = TcpListener::bind(format!("0.0.0.0:{}",port)).unwrap();
        info!("central {} listening on port {}",name, port);
        listener.set_nonblocking(true).unwrap();
        loop {
            // println!("inside the {} loop",name);
            sleep(Duration::from_millis(10));
            if stop.load(Ordering::Relaxed) == true {
                println!("CENTRAL: {} interface told to quit",name);
                break;
            }
            match listener.accept() {
                Ok((stream,add)) => {
                    println!("CENTRAL: {} thread, accepting client from {}",name,add);
                    cb(stream,tx.clone(),stop.clone(), state.clone());
                    // state.lock().unwrap().add_app_from_stream(stream.try_clone().unwrap(), tx.clone(), stop.clone());
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        // println!("need to loop again, {}",name);
                    } else {
                        println!("CENTRAL: real error {} interface quitting.  {}", name,e);
                        break;
                    }
                }
            }
        }
        drop(listener);
        println!("CENTRAL: {} thread quitting",name);
    })
}
