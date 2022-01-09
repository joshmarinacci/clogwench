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

use common::{APICommand, KeyDownEvent};

pub struct App {
    connection:TcpStream,
    pub receiver_handle: JoinHandle<()>,
}
fn main() {
    let args:Cli = init_setup();

    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());

    let (tx, rx) = mpsc::channel::<APICommand>();

    let app_list: Arc<Mutex<Vec<App>>> = Arc::new(Mutex::new(Vec::new()));

    start_network_server(stop.clone(), tx, app_list.clone());
    start_event_processor(stop.clone(), rx);
    // let ch = start_process();

    if args.keyboard {
        send_fake_keyboard(app_list.clone(), stop.clone());
    }

    let timeout_handle = start_timeout(stop.clone(), args.timeout);
    timeout_handle.join().unwrap();
    info!("all done now");
}


fn start_event_processor(stop: Arc<AtomicBool>, rx: Receiver<APICommand>) -> JoinHandle<()> {
    return thread::spawn(move || {
        for cmd in rx {
            info!("processing event {:?}",cmd);
            if stop.load(Ordering::Relaxed) { break; }
        }
    });
}

fn start_network_server(stop: Arc<AtomicBool>,
                        tx: Sender<APICommand>,
                        app_list: Arc<Mutex<Vec<App>>>) -> JoinHandle<()> {

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
                    let app = App {
                        connection: stream.try_clone().unwrap(),
                        receiver_handle:handle_client(stream.try_clone().unwrap(),tx.clone(),stop.clone()),
                    };
                    app_list.lock().unwrap().push(app);
                }
                Err(e) => {
                    error!("error: {}",e);
                }
            }
        }
        drop(listener);
    })
}

fn send_fake_keyboard(app_list: Arc<Mutex<Vec<App>>>, stop: Arc<AtomicBool>) {
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
                    let mut v = app_list.lock().unwrap();
                    for app in v.iter_mut() {
                        app.connection.write_all(data.as_ref()).expect("failed to send rect");
                    }
                }
                thread::sleep(Duration::from_millis(1000));
            }
        }
    });

}

fn handle_client(stream: TcpStream, tx: Sender<APICommand>, stop: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut de = serde_json::Deserializer::from_reader(stream);
        loop {
            if stop.load(Ordering::Relaxed) == true {
                info!("client thread stopping");
                break;
            }
            match APICommand::deserialize(&mut de) {
                Ok(cmd) => {
                    info!("server received command {:?}",cmd);
                    tx.send(cmd).unwrap();
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
