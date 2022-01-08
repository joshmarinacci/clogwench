use std::io::Read;
use std::net::{TcpListener, TcpStream};
use serde::Deserialize;
use serde_json;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use common::{APICommand, };
use ctrlc;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Cli {
    #[structopt(short, long)]
    debug:bool,
    #[structopt(short, long, default_value="10")]
    timeout:u32,
}


fn main() {
    let args:Cli = Cli::from_args();
    println!("running with args {:?}",args);

    let should_stop:Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    setup_c_handler(should_stop.clone());

    let (tx, rx) = mpsc::channel::<APICommand>();

    //setup network listener
    setup_network_listener(should_stop.clone(), tx);

    start_event_processor(should_stop.clone(),rx);
    //start a child process
    // let ch = start_process();


    let timeout = start_timeout(should_stop.clone(),args.timeout);
    timeout.join().unwrap();
    println!("all done now");

}

fn start_event_processor(stop: Arc<AtomicBool>, rx: Receiver<APICommand>) -> JoinHandle<()> {
    return thread::spawn(move || {
        for cmd in rx {
            println!("server processing event {:?}",cmd);
        }
    });
}

fn setup_network_listener(stop: Arc<AtomicBool>, tx: Sender<APICommand>) -> JoinHandle<()> {
    return thread::spawn(move || {
        println!("starting network connection");
        let port = 3333;
        let listener = TcpListener::bind(format!("0.0.0.0:{}",port)).unwrap();
        println!("server listening on port {}",port);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("got a new connection");
                    let txx = tx.clone();
                    thread::spawn(move||{
                        handle_client(stream,txx);
                    });
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
fn handle_client(mut stream: TcpStream, tx: Sender<APICommand>) {
    let mut de = serde_json::Deserializer::from_reader(stream);
    loop {
        // if arc.load(Ordering::Relaxed) == true {
        //     println!("socket thread stopping");
        //     break;
        // }
        println!("server reading from socket");
        let cmd:APICommand =APICommand::deserialize(&mut de).unwrap();
        println!("server is getting results {:?}",cmd);
        tx.send(cmd).unwrap();
    }
    // let mut data = [0 as u8; 500];
    // while match stream.read_to_end(&mut data) {
    //     Ok(size) => {
    //         println!("got a message");
    //         true
    //     },
    //     Err(_) => {
    //         println!("");
    //         false
    //     }
    // } {}
}

fn start_timeout(stop: Arc<AtomicBool>, max_seconds:u32) -> JoinHandle<()> {
    return thread::spawn(move || {
        println!("timeout will end in {} seconds",max_seconds);
        let mut count = 0;
        loop {
            count = count + 1;
            if count > max_seconds {
                stop.store(true,Ordering::Relaxed);
            }
            // println!("watchdog sleeping for 1000 {}",count);
            thread::sleep(Duration::from_millis(1000));

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
