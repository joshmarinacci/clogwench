use interprocess::local_socket::LocalSocketStream;
use std::io::{self, prelude::*, BufReader};
use std::thread;
use std::time::Duration;
use common::{APICommand, ARGBColor, DrawRectCommand, OpenWindowCommand};
use serde::{Deserialize, Serialize};
use rand::prelude::*;


fn main()  {
    let mut rng = rand::thread_rng();
    let mut conn = BufReader::new(
        LocalSocketStream::connect("/tmp/teletype.sock").expect("failed to connect"),
    );
    eprintln!("Teletype client connected to server.");
    let mut our_turn = true;
    let mut buffer = String::new();
    let mut count = 0;
    loop {
        count = count + 1;
        let cmd = if count % 2 == 0 {
            APICommand::DrawRectCommand(DrawRectCommand{
                x: rng.gen_range(0..100),
                y: rng.gen_range(0..100),
                w: 100,
                h: 100,
                color: ARGBColor {
                    r: rng.gen_range(0..255),
                    g: rng.gen_range(0..255),
                    b: rng.gen_range(0..255),
                    a: 255
                }
            })
        } else {
            APICommand::OpenWindowCommand(OpenWindowCommand{
                name: 0
            })
        };
        let data = serde_json::to_string(&cmd).unwrap();
        conn.get_mut().write_all(data.as_ref()).expect("failed to send rect");
        // conn.get_mut().write(b"\n");
        println!("sent data {:?}",data);
        thread::sleep(Duration::from_millis(1000));
        buffer.clear();
        our_turn = !our_turn;
    }
}
