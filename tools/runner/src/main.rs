use structopt::StructOpt;
mod headlesswm;
mod central_conn;
mod platwm;


use std::path::PathBuf;
use std::process::{Child, Command};
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::spawn;
use std::time::Duration;
use log::{info, LevelFilter, set_logger};
use serde::Deserialize;
use uuid::Uuid;
use common::{APICommand, DebugMessage, HelloWindowManager, IncomingMessage, WINDOW_MANAGER_PORT};
use cool_logger::CoolLogger;
use gfx::graphics::Rect;
use crate::headlesswm::HeadlessWindowManager;
use crate::platwm::{PlatformWindowManager};

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
    #[structopt(short, long)]
    test:bool,
    // #[structopt(long)]
    // start_clock:bool,
    // #[structopt(long)]
    // start_echo:bool,
    // #[structopt(long)]
    // start_dock:bool,
    #[structopt(long, default_value="1")]
    scale:u32,
    #[structopt(long, default_value="640")]
    width:u32,
    #[structopt(long, default_value="480")]
    height:u32,
    #[structopt(long, parse(from_os_str))]
    datafile: Vec<PathBuf>,
}

fn init_setup() -> Cli {
    let args: Cli = Cli::from_args();
    return args;
}


static COOL_LOGGER:CoolLogger = CoolLogger;
fn main() -> Result<(),String> {
    let args:Cli = init_setup();
    set_logger(&COOL_LOGGER).map(|()|log::set_max_level(LevelFilter::Info));

    // start central server
    let mut debug_channel = central_conn::start_central_server(&args.datafile)?;
    // info!("runner: connected to the central server");
    debug_channel.send(DebugMessage::HelloDebugger);
    // info!("runner: sent the hello debugger message");
    debug_channel.wait_for(DebugMessage::HelloDebuggerResponse);
    // info!("runner: got back the response!");

    // wait(1000);

    if args.test {
        let test_handler = spawn({
            move || {
                // wait for Debug::window_manager_connected
                info!("test: test thread waiting for window manager connected");
                debug_channel.wait_for(DebugMessage::WindowManagerConnected);

                // wait(4000);
                info!("test: starting the app");

                // start demo click grid. opens window at 50,50 to 250,250
                let mut app_thread = start_app("demo-click-grid");
                // wait for the app to start
                debug_channel.wait_for(DebugMessage::AppConnected(String::from("demo-click-grid")));
                info!("test: app connected");

                // send wait for the window to open
                debug_channel.wait_for(DebugMessage::WindowOpened(String::from("demo-click-grid")));
                info!("test: app window open");
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

                //request a screen capture
                debug_channel.send(DebugMessage::ScreenCapture(Rect::from_ints(0, 0, 500, 500), String::from("path.png")));
                debug_channel.wait_for(DebugMessage::ScreenCaptureResponse());
                info!("waiting 5 seconds");
                wait(5000);
                info!("RUNNER: killing the central server");
                debug_channel.send(DebugMessage::RequestServerShutdown);
                wait(5000);
                info!("sending a process kill in case its still running");
                debug_channel.child.kill().unwrap();

                app_thread.child.kill().unwrap();
            }
        });
    } else {
        info!("Lets just dump debug messages instead of running a test");
        let test_handler = spawn(||{
            info!("monitoring the debug log");
            debug_channel.loop_until_done();
        });

    }


    match args.wmtype {
        WMType::Native => {
            info!("creating a native window manager");
            let mut wm = PlatformWindowManager::init(args.width, args.height, args.scale).unwrap();
            wm.make_fake_window(&"fake window 1".to_string(),&Rect::from_ints(100,100,150,200));
            wm.make_fake_window(&"fake window 2".to_string(),&Rect::from_ints(300,100,150,200));
            {
                loop {
                    let keep_going = wm.main_service_loop();
                    if !keep_going { break; }
                }
                info!("WM Native shutting down");
                wm.shutdown();
            }
        }
        WMType::Headless => {
            info!("creating a headless window manager");
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

    info!("waiting for the test handler to finish");
    // test_handler.join();
    info!("runner fully done");

    Ok(())
}

fn start_app_with_delay(delay: u64, path: String) {
    thread::spawn(move||{
        thread::sleep(Duration::from_millis(delay));
        info!("launching {}",path);
        let mut child = Command::new("cargo")
            .current_dir(&path)
            // .stdin(Stdio::null())
            // .stdout(Stdio::null())
            // .stdout(Stdio::inherit())
            .arg("run")
            // .arg("--debug=true")
            // .env_clear()
            // .env("PATH", "/bin")
            .spawn()
            .expect("child process failed to start")
            ;
        info!("child at {} is launched",path);
    });
}

fn start_nodeapp_with_delay(delay: u64, path: String) {
    thread::spawn(move||{
        thread::sleep(Duration::from_millis(delay));
        info!("launching {}",path);
        let mut child = Command::new("npm")
            .current_dir(&path)
            // .stdin(Stdio::null())
            // .stdout(Stdio::null())
            // .stdout(Stdio::inherit())
            .arg("run")
            .arg("start")
            // .arg("--debug=true")
            // .env_clear()
            // .env("PATH", "/bin")
            .spawn()
            .expect("child process failed to start")
            ;
        info!("child at {} is launched",path);
    });
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
    let mut child = Command::new(path)
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
