use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc::{Receiver, Sender};
use common::{DEBUG_PORT, DebugMessage};
use std::process::{Child, Command};
use std::net::TcpStream;
use std::sync::mpsc;
use common::events::MouseDownEvent;
use serde::{Deserialize, Deserializer};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use log::{error, info};
use serde_json::{Map, Value};

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
                    Ok(cmd.clone())
                } else {
                    info!("incorrect message!");
                    Ok(cmd.clone())
                }
            }
            Err(e) => {
                info!("error deserializing {:?}", e);
                Err(e.to_string())
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

pub fn start_central_server(files: &Vec<PathBuf>) -> Result<CentralConnection,String> {
    let (sender,receiver):(Sender<DebugMessage>,Receiver<DebugMessage>) = mpsc::channel();
    info!("using the datafile path {:?}",files);
    let mut data_out:Vec<Value> = vec![];

    for file in files {
        info!("Loading data file {:?}",file.canonicalize().unwrap());
        let file = File::open(file).unwrap();
        let val:Value = serde_json::from_reader(BufReader::new(file)).unwrap();
        let data_field = val.as_object().unwrap().get("data");
        let data_part = data_field.unwrap().as_array().unwrap();
        for val in data_part {
            info!("adding {:?}",val.as_object().unwrap().get("id").unwrap().as_str().unwrap());
            data_out.push(val.clone());
        }
    }
    let mut hm:Map<String,Value> = Map::new();
    hm.insert(String::from("data"), Value::Array(data_out));
    let object_out:Value = Value::Object(hm);

    let final_datafile = "data_combined.json";
    let output = File::create(final_datafile).unwrap();
    serde_json::to_writer(output,&object_out).unwrap();

    let child = Command::new("../../target/debug/central")
        // .stdin(Stdio::null())
        // .stdout(Stdio::null())
        // .stdout(Stdio::inherit())
        .arg("--debug=true")
        .arg(format!("--database={}", final_datafile))
        .env_clear()
        // .env("PATH", "/bin")
        .spawn()
        .expect("child process failed to start")
        ;
    info!("started CENTRAL process");

    loop {
        let conn_string = format!("localhost:{}", DEBUG_PORT);
        match TcpStream::connect(conn_string) {
            Ok(master_stream) => {
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
