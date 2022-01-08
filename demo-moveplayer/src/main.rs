use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{IntoIter, Iter, Receiver, Sender, TryIter};
use std::thread;
use serde::Deserialize;
use common::{APICommand, ARGBColor, DrawRectCommand, OpenWindowCommand};

struct ClientConnection {
    stream: TcpStream,
    tx: Sender<APICommand>,
    rx: Receiver<APICommand>,
}

impl ClientConnection {
    pub(crate) fn send(&self, cmd: APICommand) {
        self.tx.send(cmd).unwrap();
    }
}

impl ClientConnection {
    pub fn init() -> Option<ClientConnection> {
        let (in_tx, in_rx) = mpsc::channel::<APICommand>();
        let (out_tx, out_rx) = mpsc::channel::<APICommand>();
        match TcpStream::connect("localhost:3333") {
            Ok(stream) => {
                println!("connected to the server");
                let mut stream1 = stream.try_clone().unwrap();
                let stream2 = stream.try_clone().unwrap();

                //receiving thread
                let hand2 = thread::spawn(move||{
                    println!("receiving thread starting");
                    let mut de = serde_json::Deserializer::from_reader(stream2);
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
                });
                //sending thread
                let hand3 = thread::spawn(move|| {
                    println!("sending thread starting");
                    for cmd in out_rx {
                        let data = serde_json::to_string(&cmd).unwrap();
                        println!("sending data {:?}",data);
                        stream1.write_all(data.as_ref()).expect("failed to send rect");
                        // thread::sleep(Duration::from_millis(1000));
                    }
                });
                Some(ClientConnection {
                    stream,
                    tx:out_tx,
                    rx:in_rx,
                })
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                None
            }
        }
    }
}
fn redraw(client: &ClientConnection, x: i32, y: i32, w:i32, h:i32) {
    let white:ARGBColor = ARGBColor{
        r: 255,
        g: 255,
        b: 255,
        a: 255
    };
    let black:ARGBColor = ARGBColor{
        r: 0,
        g: 0,
        b: 0,
        a: 255
    };

    //draw background and wait
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        x:0, y:0, w, h, color: white,
    }));
    //draw player and wait
    client.send(APICommand::DrawRectCommand(DrawRectCommand{
        x, y, w:10, h:10, color: black,
    }));
}
fn main() {
    /*
    window is 100x100
    set player position at 50
    client sends open window. wait.
    client sends draw rect to fill the window. wait.
    when client receives a keyboard. wait
        update player position.
        client sends draw rect event. wait.
     */

    let w = 100;
    let h = 100;
    let mut x = 50;
    let mut y = 50;

    let client = ClientConnection::init().expect("Can't connect to the server");
    //open window and wait
    client.send(APICommand::OpenWindowCommand(OpenWindowCommand{ name: 0 }));
    redraw(&client,x,y,w,h);

    for cmd in &client.rx {
        println!("got an event {:?}",cmd);
        match cmd {
            APICommand::KeyDown(kd) => {
                println!("got a keydown event");
                x += 1;
                redraw(&client,x,y,w,h)
            }
            _ => {}
        }
    }

}
