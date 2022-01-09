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

use common::{APICommand, App, CentralState, IncomingMessage, KeyDownEvent, Window};

fn main() {
    let args:Cli = init_setup();

    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    let (tx, rx) = mpsc::channel::<IncomingMessage>();

    let state:Arc<Mutex<CentralState>> = Arc::new(Mutex::new(CentralState::init()));

    start_network_server(stop.clone(), tx, state.clone());
    start_event_processor(stop.clone(), rx, state.clone());
    // let ch = start_process();

    if args.keyboard {
        send_fake_keyboard(state.clone(), stop.clone());
    }

    let timeout_handle = start_timeout(stop.clone(), args.timeout);
    timeout_handle.join().unwrap();
    info!("all done now");
}


fn start_event_processor(stop: Arc<AtomicBool>,
                         rx: Receiver<IncomingMessage>,
                         state: Arc<Mutex<CentralState>>
) -> JoinHandle<()> {
    return thread::spawn(move || {
        for cmd in rx {
            if stop.load(Ordering::Relaxed) { break; }
            info!("processing event {:?}",cmd);
            match cmd.command {
                APICommand::OpenWindowCommand(ow) => {
                    info!("adding a window to the app");
                    let win = Window::from_rect(ow.bounds);
                    state.lock().unwrap().add_window(cmd.appid,win);
                }
                _ => {}
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
        info!("server listening on port {}",port);
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
                    key: 1
                });
                let data = serde_json::to_string(&cmd).unwrap();
                info!("sending fake event {:?}",cmd);
                {
                    for app in state.lock().unwrap().app_list() {
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
                info!("client thread stopping");
                break;
            }
            match APICommand::deserialize(&mut de) {
                Ok(cmd) => {
                    //info!("server received command {:?}",cmd);
                    let im = IncomingMessage { appid, command:cmd, };
                    tx.send(im).unwrap();
                }
                Err(e) => {
                    error!("error deserializing from client {:?}",e);
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
#[structopt(name = "test-server", about = "simulates receiving and sending server events")]
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
