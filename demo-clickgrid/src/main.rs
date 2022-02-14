use std::io::{prelude::*};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use common::{APICommand, OpenWindowCommand, Rect};
use serde::Deserialize;

fn l(line:&str) {
    println!("DEMO_CLICK_GIRD: {}",line);
}
fn main()  {
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            l("connected to the linux-wm");
            let mut stream1 = stream.try_clone().unwrap();
            let hand1 = thread::spawn(move||{
                l("sending thread starting");
                let mut rng = rand::thread_rng();
                for i in 0..5 {
                    let cmd = APICommand::OpenWindowCommand(OpenWindowCommand{
                        window_type: "plain".to_string(),
                        bounds: Rect::from_ints(0,0,20,30),
                    });
                    let data = serde_json::to_string(&cmd).unwrap();
                    // l(&format!("sending data {:?}",data));
                    stream1.write_all(data.as_ref()).expect("failed to send rect");
                    thread::sleep(Duration::from_millis(1000));
                }
            });
            let hand2 = thread::spawn(move||{
                l("receiving thread starting");
                let mut de = serde_json::Deserializer::from_reader(stream);
                loop {
                    match APICommand::deserialize(&mut de) {
                        Ok(cmd) => {
                            println!("demo-clickgrid received command {:?}", cmd);
                        }
                        Err(e) => {
                            println!("error deserializing from demo-clickgrid {:?}", e);
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
