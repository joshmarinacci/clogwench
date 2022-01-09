pub mod client {
    use std::sync::mpsc;
    use std::net::TcpStream;
    use std::thread;
    use std::sync::mpsc::{Receiver, Sender};
    use serde::Deserialize;
    use std::io::Write;
    use crate::APICommand;

    pub struct ClientConnection {
        stream: TcpStream,
        tx: Sender<APICommand>,
        pub rx: Receiver<APICommand>,
    }

    impl ClientConnection {
        pub fn send(&self, cmd: APICommand) {
            self.tx.send(cmd).unwrap();
        }
    }

    impl ClientConnection {
        pub fn init() -> Option<ClientConnection> {
            let (in_tx, in_rx) = mpsc::channel::<APICommand>();
            let (out_tx, out_rx) = mpsc::channel::<APICommand>();
            match TcpStream::connect("localhost:3333") {
                Ok(master_stream) => {
                    println!("connected to the server");

                    //receiving thread
                    let receiving_handle = thread::spawn({
                        let mut stream = master_stream.try_clone().unwrap();
                        move || {
                            println!("receiving thread starting");
                            let mut de = serde_json::Deserializer::from_reader(stream);
                            loop {
                                match APICommand::deserialize(&mut de) {
                                    Ok(cmd) => {
                                        println!("client received command {:?}", cmd);
                                        in_tx.send(cmd);
                                    }
                                    Err(e) => {
                                        println!("error deserializing from client {:?}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    });
                    //sending thread
                    let sending_handle = thread::spawn({
                        let mut stream = master_stream.try_clone().unwrap();
                        move || {
                            println!("sending thread starting");
                            for cmd in out_rx {
                                let data = serde_json::to_string(&cmd).unwrap();
                                println!("sending data {:?}", data);
                                stream.write_all(data.as_ref()).expect("failed to send rect");
                            }
                        }
                    });
                    Some(ClientConnection {
                        stream: master_stream,
                        tx: out_tx,
                        rx: in_rx,
                    })
                }
                Err(e) => {
                    println!("Failed to connect: {}", e);
                    None
                }
            }
        }
    }
}
