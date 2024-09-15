use zmq;
use std::{str, thread};

fn main() {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::DEALER).unwrap();
    socket.connect("tcp://127.0.0.1:3000").unwrap();

    println!("app connected");
    loop {
        if socket.poll(zmq::POLLIN, 10).expect("client failed polling") > 0 {
            let msg = socket
                .recv_multipart(0)
                .expect("client failed receiving response");
            println!("got {}", str::from_utf8(&msg[msg.len()-1]).unwrap());
        }
    }

    // socket.send("hello world!", 0).unwrap();
}

