use std::net::{TcpListener, TcpStream};
use std::thread::JoinHandle;
use common::{APICommand, KeyDownEvent};
use serde::Deserialize;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use log::{info, warn, error,log};

pub fn start_network_server(stop:Arc<AtomicBool>, tx:Sender<APICommand>) -> JoinHandle<()> {
    thread::spawn(move||{
        info!("starting network connection");
        let port = 3333;
        let listener = TcpListener::bind(format!("0.0.0.0:{}",port)).expect("Couldn't bind to port");
        info!("server listening on port {}",port);
        for stream in listener.incoming() {
            if stop.load(Ordering::Relaxed) { break; }
            match stream {
                Ok(stream) => {
                    handle_client(stream.try_clone().unwrap(),stop.clone(),tx.clone());
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