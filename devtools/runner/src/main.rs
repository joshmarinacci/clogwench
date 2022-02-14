use structopt::StructOpt;
mod headlesswm;
mod central_conn;
mod platwm;


use std::fmt::DebugList;
use std::io::Write;
use std::net::TcpStream;
use std::process::{Child, Command};
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::spawn;
use std::time::Duration;
use serde::Deserialize;
use common::{APICommand, DEBUG_PORT, DebugMessage, HelloWindowManager, IncomingMessage, Point, Rect, WINDOW_MANAGER_PORT};
use common::events::MouseDownEvent;
use common_wm::{OutgoingMessage, WindowManagerState};
use crate::headlesswm::HeadlessWindowManager;
use crate::platwm::{main_service_loop, PlatformWindowManager};

#[derive(Debug, Copy, Clone, Deserialize)]
enum WMType {
    Native,
    Headless
}

type ParseError = &'static str;

impl FromStr for WMType {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "native" => Ok(WMType::Native),
            "headless" => Ok(WMType::Headless),
            _ => Err("Could not parse a day"),
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "dev-runner", about = "start idealos with specific config")]
struct Cli {
    #[structopt(long)]
    wmtype: WMType,
}

fn init_setup() -> Cli {
    let args: Cli = Cli::from_args();
    return args;
}

fn main() -> Result<(),String> {
    let args:Cli = init_setup();
    println!("Hello, world!");

    // start central server
    let mut debug_channel = central_conn::start_central_server()?;
    println!("runner: connected to the central server");
    debug_channel.send(DebugMessage::HelloDebugger);
    println!("runner: sent the hello debugger message");
    debug_channel.wait_for(DebugMessage::HelloDebuggerResponse);
    println!("runner: got back the response!");

    wait(1000);

    let test_handler = spawn({
        move || {
            // wait for Debug::window_manager_connected
            println!("test thread waiting for window manager connected");
            debug_channel.wait_for(DebugMessage::WindowManagerConnected);

            // wait(4000);
            println!("runner: starting the app");

            // start demo click grid. opens window at 50,50 to 250,250
            let app_thread = start_app("demo-click-grid");
            // wait for debug::app_started(name === name passed to demo click grid)
            debug_channel.wait_for(DebugMessage::AppConnected(String::from("demo-click-grid")));
            println!("RUNNER: got the message app connected?");
            // send for debug::window_opened(app name == name passed to demo click grid)
            // debug_channel.wait_for(DebugMessage::WindowOpened(String::from("demo-click-grid")));
            // wait(1000);
            // send fake click to the background
            // debug_channel.send_mouse_event(MouseDownEvent::init_primary(600,500));
            // wait for debug::background received click
            // debug_channel.wait_for(DebugMessage::BackgroundReceivedMouseEvent);
            // send fake click to window
            // debug_channel.send_mouse_event(MouseDownEvent::init_primary(200,200));
            // wait for debug::focused window changed, appname == name passed to demo click grid)
            // debug_channel.wait_for(DebugMessage::WindowFocusChanged(String::from("demo-click-grid")));
            // app receives click. sends out a debug log event saying it got a click
            // wait for debug log event from that appname.
            // debug_channel.wait_for(DebugMessage::AppLog(String::from("input-received")));

            // wait(1000);
            /*
            debug_channel.send(DebugMessage::ScreenCapture(Rect::from_ints(0,0,500,500),String::from("path.png")));
            debug_channel.wait_for(DebugMessage::ScreenCaptureResponse());
             */
            println!("waiting 5 seconds");
            wait(5000);
            println!("killing the central server");
            debug_channel.send(DebugMessage::RequestServerShutdown);
            wait(5000);
            println!("sending a kill in case its still running");
            debug_channel.child.kill().unwrap();
        }
    });


    // start window manager
    println!("the test thread is going. now lets start the window manager on the main thread");

    match args.wmtype {
        WMType::Native => {
            println!("creating a native window manager");
            let mut wm = PlatformWindowManager::init(800, 800).unwrap();
            //send the initial hello message
            let im = OutgoingMessage {
                recipient: Default::default(),
                command: APICommand::WMConnect(HelloWindowManager {})
            };
            wm.tx_out.send(im).unwrap();
            {
                loop {
                    let keep_going = main_service_loop(&mut wm.state, &mut wm.plat, &mut wm.rx_in, &mut wm.tx_out);
                    if !keep_going { break; }
                }
                println!("RUNNER: WM Native shutting down");
                wm.plat.shutdown();
            }
            println!("RUNNER: WM Native shut down");
            // pt("window manager fully connected to the central server");

        }
        WMType::Headless => {
            println!("creating a headless window manager");
            let wm = HeadlessWindowManager::init(800,800).unwrap();
            wm.handle.join();
        }
    }


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
    // debug_channel.wait_join();
    // print success
    // exit

    test_handler.join();
    println!("runner fully done");

    Ok(())
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
