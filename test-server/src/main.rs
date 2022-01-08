use std::io::Read;
use std::net::{TcpListener, TcpStream};
use socket2::{Domain, SockAddr, Socket, Type};

use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use common::{APICommand, ARGBColor, KeyDownEvent, MouseMoveEvent};
use ctrlc;

fn main() {
    let should_stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    setup_c_handler(should_stop.clone());

    let (tx, rx) = mpsc::channel::<APICommand>();

    //setup network listener
    setup_network_listener(should_stop.clone(), tx);

    //start a child process
    // let ch = start_process();


    let timeout = start_timeout(should_stop.clone());
    timeout.join().unwrap();
    println!("all done now");

}

fn setup_network_listener(stop: Arc<AtomicBool>, tx: Sender<APICommand>) -> JoinHandle<()> {
    return thread::spawn(move || {
        println!("starting network connection");
        let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
        println!("server listening on port 3333");
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("got a new connection");
                    thread::spawn(move||{
                        handle_client(stream);
                    })
                }
                Err(e) => {
                    println!("error: {}",e);
                }
            }
        }
        drop(listener);
    })
}
// adapated from
// https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo
fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50];
    while match stream.read(&mut data) {
        Ok(size) => {
            println!("got a message");
            true
        },
        Err(_) => {
            println!("");
            false
        }
    } {}
}

fn start_timeout(stop: Arc<AtomicBool>) -> JoinHandle<()> {
    return thread::spawn(move || {
        println!("timeout will end in 15 seconds");
        let mut count = 0;
        loop {
            count = count + 1;
            if count > 15 {
                stop.store(true,Ordering::Relaxed);
            }
            // println!("watchdog sleeping for 1000");
            thread::sleep(Duration::from_millis(100));

            if stop.load(Ordering::Relaxed) == true {
                // println!("render thread stopping");
                break;
            }
        }
    });

}

fn setup_c_handler(stop: Arc<AtomicBool>) {
    ctrlc::set_handler(move || {
        stop.store(true, Ordering::Relaxed)
    }).expect("error setting control C handler");
}
