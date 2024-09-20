extern crate sdl2;
use zmq;
use std::{str};
use std::time::{Duration, Instant};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use gfx::graphics::{ARGBColor, GFXBuffer, PixelLayout, Point, Rect};
use sdl2::rect::Rect as SDLRect;
use sdl2::render::TextureAccess;

fn main() {
    println!("starting window side");

    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::DEALER).unwrap();
    socket.bind("tcp://127.0.0.1:3000").unwrap();
    println!("window bound to endpoint");
    let mut msg = zmq::Message::new();


    // let window = sdl.video.createWindow({ resizable: false, width:800, height:600 })
    // let canvas = Canvas.createCanvas(window.pixelWidth,window.pixelHeight)
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let dst: SDLRect = SDLRect::new(0,0,800,600);

    let tex_creator = canvas.texture_creator();
    let mut tex = tex_creator.create_texture(
        PixelFormatEnum::RGBA8888,
        TextureAccess::Target, dst.w as u32, dst.h as u32).unwrap();
    
    canvas.copy(&tex,None,dst).unwrap();


    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let mut last:u128 = 0;
    let now = Instant::now();
    'running: loop {
        // println!("look for input events");
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    println!("quitting");
                    break 'running;
                },
                Event::MouseButtonDown { x, y , ..} => {
                    last = now.elapsed().as_millis();
                    // println!("got mouse button down {} {} {:?}",x,y, now.elapsed().as_millis());
                    let point = Point::init(x,y);
                    let point_string = serde_json::to_string(&point).unwrap();
                    // await sock.send([null,MyMessageType.Clicked,JSON.stringify(point.toJSON())])
                    socket.send_multipart(&["clicked", point_string.as_str()], 0).unwrap()
                },
                _ => {}
            }
        }
        // println!("look for messages");
        if socket.poll(zmq::POLLIN, 10).expect("client failed polling") > 0 {
            // println!("receiving data");
            // let msg = socket.recv_multipart(0).expect("failed");
            socket.recv(&mut msg,0).unwrap();
            // println!("got {}", msg.as_str().unwrap());
            if(msg.as_str().unwrap().eq("repaint")) {
                // println!("Received a repaint message");
                socket.recv(&mut msg,0).unwrap();
                // println!("size should be {}", msg.as_str().unwrap());
                socket.recv(&mut msg,0).unwrap();
                // println!("now we got the actual image frame. len {}",msg.len());
                let arr = msg.to_vec();
                let rect = sdl2::rect::Rect::new(0,0,800,600);
                let pitch:usize = (800 * 4) as usize;
                tex.update(rect, &arr, pitch).unwrap();
                let delta = now.elapsed().as_millis() - last;
                // println!("repainted {:?}",delta);
            }
        }
        canvas.copy(&tex,None,dst).unwrap();
        // canvas.set_draw_color(Color::RED);
        // canvas.fill_rect(SDLRect::new(0,0,100,100));
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

/*
    while(true) {
        const [_null, kind,opts,data] = await sock.receive()
        console.log(`received message ${_null}, ${kind}, ${opts}`)
        if(kind.toString() === MyMessageType.Repaint) {
            const size = Size.fromJSON(JSON.parse(opts.toString()))
            console.log("size",size)
            let img = new SimpleImage(size,new Uint8ClampedArray(data))
            redraw(img)
        }
    }

    await sock.bind("tcp://127.0.0.1:3000")
    console.log("window bound to port 3000")

    let window = sdl.video.createWindow({ resizable: false, width:800, height:600 })
    let canvas = Canvas.createCanvas(window.pixelWidth,window.pixelHeight)

    window.on('mouseButtonDown', async (e)=>{
        const point =  new Point(e.x,e.y)
        console.log("click sent")
        await sock.send([null,MyMessageType.Clicked,JSON.stringify(point.toJSON())])
    })
    window.on('keyDown',(e) => {
        // console.log("keydown",e)
        if (e.key === 'q' && e.super) {
            console.log('quitting')
            sock.close()
            process.exit(0)
        }
    })

    function redraw(img?:SimpleImage) {
        const ctx = canvas.getContext("2d")
        ctx.fillStyle = 'white'
        ctx.fillRect(0, 0, canvas.width, canvas.height)
        ctx.save()
        if(img) {
            // let can2  = createCanvas(img.getSize().w,img.getSize().h)
            // let ctx2 = can2.getContext("2d")
            let dt =  createImageData(img.asUint8ClampedArray(),img.getSize().w,img.getSize().h)
            // ctx2.putImageData(dt,0,0)
            // ctx.drawImage(can2,0,0)
            ctx.putImageData(dt, 0, 0)
        }
        const buffer = canvas.toBuffer('raw')
        window.render(window.pixelWidth, window.pixelHeight, window.pixelWidth * 4, 'bgra32', buffer)
        console.timeEnd("click")
    }

    redraw()
    while(true) {
        const [_null, kind,opts,data] = await sock.receive()
        console.log(`received message ${_null}, ${kind}, ${opts}`)
        if(kind.toString() === MyMessageType.Repaint) {
            const size = Size.fromJSON(JSON.parse(opts.toString()))
            console.log("size",size)
            let img = new SimpleImage(size,new Uint8ClampedArray(data))
            redraw(img)
        }
    }
}
run()

 */
