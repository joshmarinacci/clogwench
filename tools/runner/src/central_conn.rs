use std::sync::mpsc::{Receiver, Sender};
use common::{DEBUG_PORT, DebugMessage};
use std::process::{Child, Command};
use std::net::TcpStream;
use std::sync::mpsc;
use common::events::MouseDownEvent;
use serde::Deserialize;
use std::io::Write;
use std::thread;
use std::time::Duration;
use log::{error, info};

pub struct CentralConnection {
    pub receiver: Receiver<DebugMessage>,
    pub child: Child,
    pub master_stream:TcpStream,
}


impl CentralConnection {
    // pub(crate) fn wait_join(&self) {
    //     crate::wait(1000);
    // }
    pub(crate) fn send_mouse_event(&mut self, evt: MouseDownEvent) {
        self.send(DebugMessage::FakeMouseEvent(evt));
    }
    pub(crate) fn wait_for(&self, msg: DebugMessage) -> Result<DebugMessage,String> {
        info!("waiting for {:?}",msg);
        let mut de = serde_json::Deserializer::from_reader(&self.master_stream);
        match DebugMessage::deserialize(&mut de) {
            Ok(cmd) => {
                let cmd2 = cmd.clone();
                info!("received command {:?}", cmd);
                if matches!(cmd2,msg) {
                    return Ok(cmd.clone())
                } else {
                    info!("incorrect message!");
                    return Ok(cmd.clone())
                }
            }
            Err(e) => {
                info!("error deserializing {:?}", e);
                return Err(e.to_string());
            }
        }
    }
    pub(crate) fn loop_until_done(self) {
        let mut de = serde_json::Deserializer::from_reader(&self.master_stream);
        loop {
            match DebugMessage::deserialize(&mut de) {
                Ok(cmd) => {
                    let cmd2 = cmd.clone();
                    info!("received command {:?}", cmd);
                }
                Err(e) => {
                    error!("error deserializing {:?}", e);
                    break;
                }
            }
        }
    }
    pub(crate) fn send(&mut self, im:DebugMessage) {
        // let im = IncomingMessage { source: Default::default(), command: APICommand::WMConnect(HelloWindowManager {})};
        info!("sending out message {:?}", im);
        match serde_json::to_string(&im) {
            Ok(data) => {
                // println!("sending data {:?}", data);
                if let Err(e) = self.master_stream.write_all(data.as_ref()) {
                    error!("error sending data back to server {}",e);
                    // return None
                }
            }
            Err(e) => {
                error!("error serializing incoming messages {}",e);
                // return None
            }
        }
    }
}

pub fn start_central_server() -> Result<CentralConnection,String> {
    let (sender,receiver):(Sender<DebugMessage>,Receiver<DebugMessage>) = mpsc::channel();
    let mut child = Command::new("../../target/debug/central")
        // .stdin(Stdio::null())
        // .stdout(Stdio::null())
        // .stdout(Stdio::inherit())
        .arg("--debug=true")
        .arg("--database=../../data.json")
        .env_clear()
        // .env("PATH", "/bin")
        .spawn()
        .expect("child process failed to start")
        ;
    info!("started CENTRAL process");

    loop {
        let conn_string = format!("localhost:{}", DEBUG_PORT);
        match TcpStream::connect(conn_string) {
            Ok(mut master_stream) => {
                let (tx_out, rx_out) = mpsc::channel::<DebugMessage>();
                return Ok(CentralConnection {
                    receiver,
                    child,
                    master_stream,
                })
            }
            Err(e) => {
                // info!("cant connect yet. wait 10 ms");
                thread::sleep(Duration::from_millis(10));
                // return Err(e.to_string());
            }
        }
    }

}
