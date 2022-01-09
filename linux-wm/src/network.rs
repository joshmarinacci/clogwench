use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread::JoinHandle;
use common::{APICommand, IncomingMessage};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use log::{info, warn, error,log};
use uuid::Uuid;
use crate::App;

pub fn start_network_server(stop:Arc<AtomicBool>, tx:Sender<APICommand>, app_list: Arc<Mutex<Vec<App>>>) -> JoinHandle<()> {
    thread::spawn(move||{
        info!("starting network connection");
        let port = 3333;
        let listener = TcpListener::bind(format!("0.0.0.0:{}",port)).expect("Couldn't bind to port");
        info!("linux-wm listening on port {}",port);
        for stream in listener.incoming() {
            if stop.load(Ordering::Relaxed) { break; }
            match stream {
                Ok(stream) => {
                    let app = App {
                        connection: stream.try_clone().unwrap(),
                        receiver_handle: handle_client(stream.try_clone().unwrap(), stop.clone(), tx.clone()),
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

fn handle_client(stream:TcpStream, stop:Arc<AtomicBool>, tx:Sender<APICommand>) -> JoinHandle<()>{
    thread::spawn(move || {
        let mut de = serde_json::Deserializer::from_reader(stream);
        loop {
            if stop.load(Ordering::Relaxed) { break; }
            match APICommand::deserialize(&mut de) {
                Ok(cmd) => {
                    info!("linux-wm received command {:?}",cmd);
                    tx.send(cmd).unwrap();
                }
                Err(e) => {
                    error!("error deserializing from demo-clickgrid {:?}",e);
                    break;
                }
            }
        }
    })
}


#[derive(Serialize, Deserialize, Debug)]
pub struct OutgoingMessage {
    pub recipient:Uuid,
    pub command:APICommand,
}


pub struct CentralConnection {
    stream: TcpStream,
    recv_thread: JoinHandle<()>,
    send_thread: JoinHandle<()>,
    pub(crate) tx_out: Sender<OutgoingMessage>,
    pub(crate) rx_in: Receiver<IncomingMessage>,
    pub(crate) tx_in: Sender<IncomingMessage>,
}

pub fn start_wm_network_connection(stop: Arc<AtomicBool>) -> Option<CentralConnection> {
    match TcpStream::connect("localhost:3334") {
        Ok(mut master_stream) => {
            let (tx_in, rx_in) = mpsc::channel::<IncomingMessage>();
            let (tx_out, rx_out) =mpsc::channel::<OutgoingMessage>();
            println!("connected to the linux-wm");
            //receiving thread
            let receiving_handle = thread::spawn({
                let mut stream = master_stream.try_clone().unwrap();
                let stop = stop.clone();
                let tx_in = tx_in.clone();
                move || {
                    info!("receiving thread starting");
                    let mut de = serde_json::Deserializer::from_reader(stream);
                    loop {
                        if stop.load(Ordering::Relaxed) { break; }
                        match IncomingMessage::deserialize(&mut de) {
                            Ok(cmd) => {
                                // info!("received command {:?}", cmd);
                                tx_in.send(cmd);
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
                    for out in rx_out {
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
                tx_in:tx_in,
                rx_in:rx_in,
                tx_out:tx_out,
            })

        }
        _ => None
    }
}
