use interprocess::local_socket::LocalSocketStream;
use std::io::{self, prelude::*, BufReader};
use std::net::TcpStream;
use std::{env, thread};
use std::time::Duration;
use common::{APICommand, ARGBColor, DrawRectCommand, OpenWindowCommand};
use serde::{Deserialize, Serialize};
use rand::prelude::*;



fn main()  {
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("connected to the server");
            let mut stream1 = stream.try_clone().unwrap();
            let hand1 = thread::spawn(move||{
                println!("sending thread starting");
                let mut rng = rand::thread_rng();
                for i in 0..5 {
                    let cmd = APICommand::OpenWindowCommand(OpenWindowCommand{
                        name: rng.gen_range(0..5)
                    });
                    let data = serde_json::to_string(&cmd).unwrap();
                    println!("sending data {:?}",data);
                    stream1.write_all(data.as_ref()).expect("failed to send rect");
                    thread::sleep(Duration::from_millis(1000));
                }
            });
            let hand2 = thread::spawn(move||{
                println!("receiving thread starting");
                let mut de = serde_json::Deserializer::from_reader(stream);
                loop {
                    match APICommand::deserialize(&mut de) {
                        Ok(cmd) => {
                            println!("client received command {:?}", cmd);
                        }
                        Err(e) => {
                            println!("error deserializing from client {:?}", e);
                        }
                    }
                }
            });
            hand1.join().unwrap();
            hand2.join().unwrap();
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
