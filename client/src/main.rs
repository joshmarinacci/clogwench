use interprocess::local_socket::LocalSocketStream;
use std::io::{self, prelude::*, BufReader};
use std::net::TcpStream;
use std::{env, thread};
use std::time::Duration;
use common::{APICommand, ARGBColor, DrawRectCommand, OpenWindowCommand};
use serde::{Deserialize, Serialize};
use rand::prelude::*;



fn main()  {
    let args:Vec<String> = env::args().collect();
    println!("args {:?}",args);
    let delay = &args[1];
    println!("delay is {}",delay);
    let mut rng = rand::thread_rng();
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("connected to the server");
            for i in 0..5 {
                let cmd = APICommand::OpenWindowCommand(OpenWindowCommand{
                    name: rng.gen_range(0..5)
                });
                let data = serde_json::to_string(&cmd).unwrap();
                println!("sending data {:?}",data);
                stream.write_all(data.as_ref()).expect("failed to send rect");
                thread::sleep(Duration::from_millis(1000));
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
