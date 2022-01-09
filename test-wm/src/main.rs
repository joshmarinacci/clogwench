use std::collections::HashMap;
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::process::Command;
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
use serde::{Deserialize, Serialize};
use serde_json;
use structopt::StructOpt;
use uuid::Uuid;

use common::{APICommand, HelloWindowManager, IncomingMessage, Rect};
use common::APICommand::KeyDown;
use common::events::{KeyCode, KeyDownEvent, MouseDownEvent};


#[derive(Serialize, Deserialize, Debug)]
struct OutgoingMessage {
    recipient:Uuid,
    command:APICommand,
}


fn main() {
    let args:Cli = init_setup();

    let stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(stop.clone());
    let (tx_in, rx_in) = mpsc::channel::<IncomingMessage>();
    let (tx_out, rx_out) =mpsc::channel::<OutgoingMessage>();
    //make thread for incoming messages:
    let network_thread_handler = start_network_client(stop.clone(), tx_in.clone(), rx_out)
        .expect("error connecting to the central server");
    //open network connection

    //send hello window manager
    let msg = OutgoingMessage {
        recipient: Default::default(),
        command: APICommand::WMConnect(HelloWindowManager {
        })
    };
    tx_out.send(msg).unwrap();
    let resp = rx_in.recv().unwrap();
    let mut selfid = Uuid::new_v4();
    if let APICommand::WMConnectResponse(res) = resp.command {
        info!("got response back from the server {:?}",res);
        selfid = res.wm_id;
    } else {
        panic!("did not get the window manager connect response. gah!");
    }

    let watchdog = thread::spawn({
        let stop = stop.clone();
        move ||{
            info!("watchdog thread starting");
            loop {
                if stop.load(Ordering::Relaxed) {
                    info!("shutting down the network");
                    network_thread_handler.stream.shutdown(Shutdown::Both).unwrap();
                    break;
                }
                thread::sleep(Duration::from_millis(1000))
            }
            info!("watchdog thread ending");
        }
    });

    //make thread for fake incoming events. sends to the main event thread
    if args.keyboard {
        let input_thread_handler = send_fake_keyboard(stop.clone(), tx_in.clone());
    }

    //event processing thread
    let event_thread_handler = start_event_processor(stop.clone(), rx_in, tx_out.clone());
        //draw commands. can immediately draw to the fake screen
        //app added, add to own app list
        //window added, add to own app window list
        //key pressed in event thread
        //on keypress, send to app owner of focused window
        //on mouse press, maybe change the focused window
        //on mouse press, send to window under the cursor
        //can all state live on this thread?


    info!("waiting for the watch dog");
    watchdog.join();
    info!("all done now");
}

struct InternalState {
    apps:Vec<App>,
}

impl InternalState {
    pub(crate) fn find_app(&mut self, app_id: Uuid) -> Option<&mut App> {
        self.apps.iter_mut().find(|a|a.id == app_id)
    }
}

impl InternalState {
    fn init() -> InternalState {
        InternalState {
            apps: vec![]
        }
    }
}

struct App {
    id:Uuid,
    windows:Vec<Window>,
}
struct Window {
    id:Uuid,
    bounds:Rect,
}

fn start_event_processor(stop: Arc<AtomicBool>, rx: Receiver<IncomingMessage>, tx_out: Sender<OutgoingMessage>) -> JoinHandle<()> {
    return thread::spawn(move || {
        info!("event thread starting");
        let mut state = InternalState::init();
        for cmd in rx {
            if stop.load(Ordering::Relaxed) { break; }
            info!("processing event {:?}",cmd);
            match cmd.command {
                APICommand::AppConnectResponse(res) => {
                    info!("adding an app {}",res.app_id);
                    let app = App {
                        id:res.app_id,
                        windows: vec![]
                    };
                    state.apps.push(app);
                },
                APICommand::OpenWindowResponse(ow) => {
                    info!("adding a window to the app");
                    let win = Window {
                        id:ow.window_id,
                        bounds:ow.bounds,
                    };
                    if let Some(app) = state.find_app(ow.app_id) {
                        app.windows.push(win);
                    }
                },
                APICommand::DrawRectCommand(dr) => {
                    info!("drawing a rect");
                },
                APICommand::KeyDown(kd) => {
                    info!("key down");
                    //now send to a random app's random window, if any
                    if !state.apps.is_empty() {
                        let app = &state.apps[0];
                        if !app.windows.is_empty() {
                            let win = &app.windows[0];
                            let msg = OutgoingMessage {
                                recipient: app.id,
                                command: KeyDown(KeyDownEvent{
                                    app_id: app.id,
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
                APICommand::MouseDown(ku) => {
                    info!("mouse down");
                },
                APICommand::MouseMove(ku) => {
                    info!("mouse move")
                },
                APICommand::MouseUp(ku) => {
                    info!("mouse up")
                },
                _ => {}
            };
        }
        info!("event thread ending");
    });
}

struct CentralConnection {
    stream: TcpStream,
    recv_thread: JoinHandle<()>,
    send_thread: JoinHandle<()>,
}

fn start_network_client(stop: Arc<AtomicBool>,
                        in_tx: Sender<IncomingMessage>,
                        out_rx: Receiver<OutgoingMessage>) -> Option<CentralConnection> {
    match TcpStream::connect("localhost:3334") {
        Ok(mut master_stream) => {
            println!("connected to the linux-wm");
            //receiving thread
            let receiving_handle = thread::spawn({
                let mut stream = master_stream.try_clone().unwrap();
                let stop = stop.clone();
                move || {
                    info!("receiving thread starting");
                    let mut de = serde_json::Deserializer::from_reader(stream);
                    loop {
                        if stop.load(Ordering::Relaxed) { break; }
                        match IncomingMessage::deserialize(&mut de) {
                            Ok(cmd) => {
                                // info!("received command {:?}", cmd);
                                in_tx.send(cmd);
                            }
                            Err(e) => {
                                error!("error deserializing {:?}", e);
                                stop.store(true,Ordering::Relaxed);
                                break;
                            }
                        }
                    }
                    info!("receiving thread ending")
                }
            });
            //sending thread
            let sending_handle = thread::spawn({
                let mut stream = master_stream.try_clone().unwrap();
                let stop = stop.clone();
                move || {
                    info!("sending thread starting");
                    for out in out_rx {
                        if stop.load(Ordering::Relaxed) { break; }
                        let im = IncomingMessage {
                            source: Default::default(),
                            command: out.command
                        };
                        println!("sending out message {:?}",im);
                        let data = serde_json::to_string(&im).unwrap();
                        println!("sending data {:?}", data);
                        stream.write_all(data.as_ref()).expect("failed to send rect");
                    }
                    info!("sending thread ending");
                }
            });
            Some(CentralConnection {
                stream: master_stream,
                send_thread:sending_handle,
                recv_thread:receiving_handle,
            })

        }
        _ => None
    }

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
                });
                thread::sleep(Duration::from_millis(1000));
            }
        }
    });

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
    // ctrlc::set_handler(move || {
    //     error!("control C pressed. stopping everything");
    //     stop.store(true, Ordering::Relaxed)
    // }).expect("error setting control C handler");
}
