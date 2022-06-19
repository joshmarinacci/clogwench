use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::{io, thread};
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use env_logger::Env;
use log::{debug, error, info, LevelFilter, set_logger, warn};
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use serde::Deserialize;
use uuid::Uuid;
use common::{APICommand, APP_MANAGER_PORT, AppDisconnected, DBQueryRequest, DBQueryResponse, DEBUG_PORT, DebugMessage, HelloAppResponse, HelloWindowManagerResponse, IncomingMessage, OpenWindowCommand, OpenWindowResponse, Rect, WINDOW_MANAGER_PORT};
use structopt::StructOpt;
use cool_logger::CoolLogger;
use db::{JDB, JObj};

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
            stream.set_nonblocking(false).unwrap();
            let stream2 = stream.try_clone().unwrap();
            let mut de = serde_json::Deserializer::from_reader(stream);
            loop {
                if stop.load(Ordering::Relaxed) == true {
                    info!("app thread stopping");
                    break;
                }
                match APICommand::deserialize(&mut de) {
                    Ok(cmd) => {
                        // info!("central received command {:?}",cmd);
                        let im = IncomingMessage { source: appid, command:cmd, };
                        if let Err(e) = sender.send(im) {
                            error!("error sending command {}",e);
                        }
                    }
                    Err(e) => {
                        error!("error deserializing from app {:?}",e);
                        info!("sending shutdown about app {}",appid);
                        let im = IncomingMessage { source: appid, command:APICommand::AppDisconnected(AppDisconnected{ app_id: appid.clone() })};
                        if let Err(e) = sender.send(im) {
                            error!("error sending command {}",e);
                        }
                        stream2.shutdown(Shutdown::Both);
                        break;
                    }
                }
            }
            // info!("app thread ending {}",appid);
            info!("app {} thread ending",appid);
        })
    }
    fn spawn_wm_handler(&self, wm_id: Uuid, stream: TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) -> JoinHandle<()> {
        thread::spawn(move ||{
            info!("CENTRAL: WM thread starting: {}",wm_id);
            stream.set_nonblocking(false).unwrap();
            // info!("wm thread starting: {}",wm_id);
            let stream2 = stream.try_clone().unwrap();
            let mut de = serde_json::Deserializer::from_reader(stream);
            loop {
                if stop.load(Ordering::Relaxed) == true {
                    info!("wm thread stopping");
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
            info!("WM {} thread ending", wm_id);
        })
    }
    fn spawn_debugger_handler(&self, id:Uuid, stream: TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) -> JoinHandle<()>{
        thread::spawn(move ||{
            info!("debugger thread starting: {}",id);
            stream.set_nonblocking(false).unwrap();
            let stream2 = stream.try_clone().unwrap();
            let mut de = serde_json::Deserializer::from_reader(stream);
            loop {
                if stop.load(Ordering::Relaxed) == true {
                    info!("debugger thread stopping");
                    stream2.shutdown(Shutdown::Both);
                    break;
                }
                match DebugMessage::deserialize(&mut de) {
                    Ok(cmd) => {
                        info!("CENTRAL: received debugger command {:?}",cmd);
                        sender.send(IncomingMessage{source:id, command:APICommand::Debug(cmd)}).unwrap();
                    }
                    Err(e) => {
                        // if e.kind() == io::ErrorKind::WouldBlock {
                            // println!("need to loop again, {}",name);
                        // } else {
                        info!("CENTRAL: error deserializing from debugger {:?}",e);
                        stream2.shutdown(Shutdown::Both);
                        break;
                    }
                }
            }
            println!("CENTRAL: debugger thread ending: {}",id);
        })
    }

    fn send_to_app(&mut self, id:Uuid, resp: APICommand) {
        info!("sending to app {:?}",resp);
        let data = serde_json::to_string(&resp).unwrap();
        if let Some(app) = self.apps.iter_mut().find(|a|a.id == id){
            app.stream.write_all(data.as_ref()).expect("failed to send rect");
        }
    }
    fn send_to_all_apps(&mut self, resp: APICommand) {
        info!("sending to all apps {:?}",resp);
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
        // info!("sending to wm {:?}",resp);
        let im = IncomingMessage {
            source: Default::default(),
            command: resp,
        };
        let data = serde_json::to_string(&im).unwrap();
        let mut wm = self.wms.iter_mut().find(|a|a.id == id).unwrap();
        wm.stream.write_all(data.as_ref()).expect("failed to send data to wm");
    }
    fn send_to_all_wm(&mut self, resp: APICommand) {
        // info!("CENTRAL: sending to all wm {:?}",resp);
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
        info!("CENTRAL: sending to debugger {:?}",resp);
        let data = serde_json::to_string(&resp).unwrap();
        for dbg in self.debuggers.iter_mut() {
            dbg.stream.write_all(data.as_ref()).expect("CENTRAL: error sending to debugger");
        }
    }
    fn send_to_database(&mut self, req: DBQueryRequest) {
        info!("CENTRAL: sending to the database {:?}",req);
        let mut jdb = JDB {
            data: vec![]
        };
        let mut song = JObj::make();
        song.fields.insert("title".to_string(), "Catch Me I'm Falling".to_string());
        song.fields.insert("artist".to_string(), "Pretty Poison".to_string());
        song.fields.insert("album".to_string(), "Catch Me I'm Falling".to_string());
        jdb.data.push(song);
        let msg = DBQueryResponse {
            app_id: req.app_id,
            results:jdb.data,
        };
        self.send_to_app(msg.app_id,APICommand::DBQueryResponse(msg));
    }
}

static COOL_LOGGER:CoolLogger = CoolLogger;
fn main() {
    set_logger(&COOL_LOGGER).map(|()|log::set_max_level(LevelFilter::Info));
    let args:Cli = Cli::from_args();
    info!("central server starting");
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
    info!("waiting for the app interface thread to end");
    app_network_thread.join();
    info!("waiting for the wm interface thread to end");
    wm_network_thread.join();
    info!("waiting for the debug interface thread to end");
    debug_network_thread.join();
    info!("central server stopping");
}

fn start_router(stop: Arc<AtomicBool>, rx: Receiver<IncomingMessage>, state: Arc<Mutex<CentralState>>) -> JoinHandle<()> {
    thread::spawn(move||{
        info!("router thread starting");
        for msg in rx {
            // info!("incoming message {:?}",msg);
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
                        info!("CENTRAL sending out shutdown messages and waiting a second");
                        st.send_to_all_wm(APICommand::SystemShutdown);
                        st.send_to_all_apps(APICommand::SystemShutdown);
                    }
                    thread::sleep(Duration::from_millis(1000));
                    info!("CENTRAL sending stop command to all threads");
                    stop.store(true, Ordering::Relaxed);
                    thread::sleep(Duration::from_millis(1000));
                    info!("CENTRAL the server is really ending");
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
                APICommand::AppDisconnected(dis) => {
                    let resp = APICommand::AppDisconnected(dis);
                    state.lock().unwrap().send_to_all_wm(resp.clone());
                    state.lock().unwrap().send_to_debugger(DebugMessage::AppDisconnected(String::from("foo")))
                }
                APICommand::OpenWindowCommand(ow) => {
                    info!("opening window");
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
                APICommand::WindowResized(e) => {
                    state.lock().unwrap().send_to_app(e.app_id, APICommand::WindowResized(e));
                }
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
                APICommand::DrawImageCommand(cmd) => {
                    state.lock().unwrap().send_to_all_wm(APICommand::DrawImageCommand(cmd));
                },
                APICommand::DBQueryRequest(cmd) => {
                    state.lock().unwrap().send_to_database(cmd)
                }
                APICommand::DBQueryResponse(cmd) => {
                    state.lock().unwrap().send_to_app(cmd.app_id, APICommand::DBQueryResponse(cmd))
                }
                APICommand::KeyDown(e) => {
                    state.lock().unwrap().send_to_app(e.app_id, APICommand::KeyDown(e))
                }
                APICommand::MouseDown(e) => {
                    state.lock().unwrap().send_to_app(e.app_id, APICommand::MouseDown(e))
                }
                APICommand::MouseUp(e) => {
                    state.lock().unwrap().send_to_app(e.app_id, APICommand::MouseUp(e))
                }
                _ => {
                    warn!("CENTRAL: message not handled {:?}",msg);
                }
            }
        }
        info!("CENTRAL: router thread quitting");
    })
}

#[derive(StructOpt, Debug)]
#[structopt(name = "test-wm", about = "simulates receiving and sending linux-wm events")]
struct Cli {
    #[structopt(short, long)]
    debug:bool,
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
        info!("starting {} interface on port {}",name, port);
        let listener = TcpListener::bind(format!("0.0.0.0:{}",port)).unwrap();
        listener.set_nonblocking(true).unwrap();
        loop {
            // println!("inside the {} loop",name);
            sleep(Duration::from_millis(10));
            if stop.load(Ordering::Relaxed) == true {
                info!("{} interface told to quit",name);
                break;
            }
            match listener.accept() {
                Ok((stream,add)) => {
                    info!("{} thread, accepting client from {}",name,add);
                    cb(stream,tx.clone(),stop.clone(), state.clone());
                    // state.lock().unwrap().add_app_from_stream(stream.try_clone().unwrap(), tx.clone(), stop.clone());
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        // println!("need to loop again, {}",name);
                    } else {
                        info!("real error {} interface quitting.  {}", name,e);
                        break;
                    }
                }
            }
        }
        drop(listener);
        info!("{} thread quitting",name);
    })
}
