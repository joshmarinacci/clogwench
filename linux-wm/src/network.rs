use std::io::Write;
use std::net::TcpStream;
use std::thread::JoinHandle;
use common::{APICommand, IncomingMessage};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use log::{info, error, log};
use uuid::Uuid;

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
