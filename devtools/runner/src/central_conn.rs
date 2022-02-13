use std::sync::mpsc::{Receiver, Sender};
use common::{DEBUG_PORT, DebugMessage};
use std::process::{Child, Command};
use std::net::TcpStream;
use std::sync::mpsc;
use common::events::MouseDownEvent;
use serde::Deserialize;
use std::io::Write;

pub struct CentralConnection {
    pub receiver: Receiver<DebugMessage>,
    pub child: Child,
    pub master_stream:TcpStream,
}


impl CentralConnection {
    pub(crate) fn wait_join(&self) {
        crate::wait(1000);
    }
    pub(crate) fn send_mouse_event(&mut self, evt: MouseDownEvent) {
        self.send(DebugMessage::FakeMouseEvent(evt));
    }
    pub(crate) fn wait_for(&self, msg: DebugMessage) {
        println!("RUNNER: waiting for {:?}",msg);
        let mut de = serde_json::Deserializer::from_reader(&self.master_stream);
        match DebugMessage::deserialize(&mut de) {
            Ok(cmd) => {
                println!("RUNNER: received command {:?}", cmd);
                if matches!(cmd,msg) {
                    println!("they match!");
                } else {
                    println!("incorrect message!");
                }
            }
            Err(e) => {
                println!("error deserializing {:?}", e);
            }
        }
    }
    pub(crate) fn send(&mut self, im:DebugMessage) {
        // let im = IncomingMessage { source: Default::default(), command: APICommand::WMConnect(HelloWindowManager {})};
        println!("RUNNER: sending out message {:?}", im);
        match serde_json::to_string(&im) {
            Ok(data) => {
                println!("sending data {:?}", data);
                if let Err(e) = self.master_stream.write_all(data.as_ref()) {
                    println!("error sending data back to server {}",e);
                    // return None
                }
            }
            Err(e) => {
                println!("error serializing incoming messages {}",e);
                // return None
            }
        }


    }
}

pub fn start_central_server() -> Option<CentralConnection> {
    let (sender,receiver):(Sender<DebugMessage>,Receiver<DebugMessage>) = mpsc::channel();
    let mut child = Command::new("../../target/debug/central")
        // .stdin(Stdio::null())
        // .stdout(Stdio::null())
        // .stdout(Stdio::inherit())
        .arg("--debug=true")
        .env_clear()
        // .env("PATH", "/bin")
        .spawn()
        .expect("child process failed to start")
        ;
    println!("child started");

    crate::wait(1000);

    println!("connecting to the debug port");
    let conn_string = format!("localhost:{}",DEBUG_PORT);
    match TcpStream::connect(conn_string) {
        Ok(mut master_stream) => {
            let (tx_out, rx_out) = mpsc::channel::<DebugMessage>();
            // self.master_stream = master_stream;
            return Some(CentralConnection {
                receiver,
                child,
                master_stream,
            })
        }
        Err(e) => {
            println!("error connecting to the central server {:?}",e);
            return None
        }
    }

}
