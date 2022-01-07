use interprocess::local_socket::LocalSocketStream;
use std::io::{self, prelude::*, BufReader};
use std::thread;
use std::time::Duration;

use common::DrawRectCommand;

use serde::{Deserialize, Serialize};


fn main()  {
    let mut conn = BufReader::new(
        LocalSocketStream::connect("/tmp/teletype.sock").expect("failed to connect"),
    );
    eprintln!("Teletype client connected to server.");
    let mut our_turn = true;
    let mut buffer = String::new();
    loop {
        // if our_turn {
            let dr = DrawRectCommand {
                x: 1,
                y: 2,
                w: 3,
                h: 4
            };
            println!("rect is {:?}",dr);
            let data = serde_json::to_string(&dr).unwrap();
            conn.get_mut().write_all(data.as_ref()).expect("failed to send rect");
            conn.get_mut().write(b"\n");
            thread::sleep(Duration::from_millis(1000));

            // io::stdin()
            //     .read_line(&mut buffer)
            //     .expect("failed to read line from stdin");
            // conn.get_mut()
            //     .write_all(buffer.as_ref())
            //     .expect("failed to write line to socket");
        // } else {
            // conn.read_line(&mut buffer)
            //     .expect("failed to read line from socket");
            // io::stdout()
            //     .write_all(buffer.as_ref())
            //     .expect("failed to write line to stdout");
        // }
        buffer.clear();
        our_turn = !our_turn;
    }
}
