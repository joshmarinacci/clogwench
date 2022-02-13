mod headlesswm;

use std::fmt::DebugList;
use std::io::Write;
use std::net::TcpStream;
use std::process::{Child, Command};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use serde::Deserialize;
use common::{APICommand, DEBUG_PORT, DebugMessage, HelloWindowManager, IncomingMessage, Point, WINDOW_MANAGER_PORT};
use common::events::MouseDownEvent;
use common_wm::{OutgoingMessage, WindowManagerState};
use crate::headlesswm::HeadlessWindowManager;


pub struct CentralConnection {
    pub receiver: Receiver<DebugMessage>,
    pub child: Child,
    pub master_stream:TcpStream,
}


impl CentralConnection {
    pub(crate) fn wait_join(&self) {
        wait(1000);
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

fn main() {
    println!("Hello, world!");

    // start central server
    let mut debug_channel = start_central_server().unwrap();
    println!("runner: connected to the central server");
    debug_channel.send(DebugMessage::HelloDebugger);
    println!("runner: sent the hello debugger message");
    debug_channel.wait_for(DebugMessage::HelloDebuggerResponse);
    println!("runner: got back the response!");
    // wait for Debug::server_started
    // debug_channel.wait_for(DebugMessage::ServerStarted);

    wait(1000);
    // start window manager
    let wm = HeadlessWindowManager::init().unwrap();
    // wait for Debug::window_manager_connected
    debug_channel.wait_for(DebugMessage::WindowManagerConnected);

    wait(1000);
    println!("runner: starting the app");

    // start demo click grid. opens window at 50,50 to 250,250
    let app_thread = start_app("demo-click-grid");
    // wait for debug::app_started(name === name passed to demo click grid)
    debug_channel.wait_for(DebugMessage::AppConnected(String::from("demo-click-grid")));
    println!("got the message app connected?");
    // send for debug::window_opened(app name == name passed to demo click grid)
    debug_channel.wait_for(DebugMessage::WindowOpened(String::from("demo-click-grid")));
    wait(1000);
    // send fake click to the background
    debug_channel.send_mouse_event(MouseDownEvent::init_primary(600,500));
    // wait for debug::background received click
    debug_channel.wait_for(DebugMessage::BackgroundReceivedMouseEvent);
    // send fake click to window
    debug_channel.send_mouse_event(MouseDownEvent::init_primary(200,200));
    // wait for debug::focused window changed, appname == name passed to demo click grid)
    debug_channel.wait_for(DebugMessage::WindowFocusChanged(String::from("demo-click-grid")));
    // app receives click. sends out a debug log event saying it got a click
    // wait for debug log event from that appname.
    debug_channel.wait_for(DebugMessage::AppLog(String::from("input-received")));

    wait(1000);

    println!("killing the central server");
    debug_channel.child.kill().unwrap();

    /*
    // send kill signal to app thread
    app_thread.send_kill();
    // wait for app handle to join
    app_thread.join();
    // wait for debug::window closed event
    debug_channel.wait_for(DebugMessage::WindowClosed(String::from("demo-click-grid")));
    // wait for debug::app exit event
    debug_channel.wait_for(DebugMessage::AppDisconnected(String::from("demo-click-grid")));
    // send shutdown message to central server
     */
    // debug_channel.send(DebugMessage::RequestServerShutdown);
    // wait for window manager exited event
    // debug_channel.wait_for(DebugMessage::WindowManagerDisconnected);
    // wait for system ending event
    // debug_channel.wait_for(DebugMessage::ServerStopped);
    // wait for system handle to join
    debug_channel.wait_join();
    // print success
    println!("test app is a success");
    // exit

}

struct ChildProxy {
    child: Child,
}

impl ChildProxy {
    pub(crate) fn send_kill(&self) {
        println!("killing child process");
        wait(1000);
    }
}

impl ChildProxy {
    pub(crate) fn join(&self) {
        wait(1000);
    }
}

fn start_app(path: &str) -> ChildProxy {
    let (sender,receiver):(Sender<DebugMessage>,Receiver<DebugMessage>) = mpsc::channel();
    let mut child = Command::new("../../target/debug/demo-moveplayer")
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
    ChildProxy {
        child:child,
    }
}


fn wait(msec: i32) {
    thread::sleep(Duration::from_millis(msec as u64));
}

fn start_central_server() -> Option<CentralConnection> {
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

    wait(1000);

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
