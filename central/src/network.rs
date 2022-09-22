use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use common::{APICommand, AppDisconnected, DebugMessage, IncomingMessage};
use std::thread::{JoinHandle, sleep};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::{io, thread};
use std::time::Duration;
use log::{error, info};
use uuid::Uuid;
use serde::Deserialize;
use crate::state::CentralState;

pub fn setup_interface<F>(stop: Arc<AtomicBool>,
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

pub fn spawn_client_handler(uuid: Uuid, stream: TcpStream, sender: Sender<IncomingMessage>, stop: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::spawn(move ||{
        info!("CENTRAL: client thread starting: {}",uuid);
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
            // read IncomingMessage from stream, convert to IncomingMessage, then send to sender
            match IncomingMessage::deserialize(&mut de) {
                Ok(cmd) => {
                    // info!("central received wm command {:?}",cmd);
                    sender.send(IncomingMessage{ source: uuid, command: cmd.command }).unwrap();
                }
                Err(e) => {
                    error!("error deserializing from window manager {:?}",e);
                    stream2.shutdown(Shutdown::Both);
                    break;
                }
            }
        }
        info!("client {} thread ending", uuid);
    })
}
