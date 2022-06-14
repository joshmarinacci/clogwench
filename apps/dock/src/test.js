import {Socket} from "net"

class Window {
    constructor(app,info) {
        this.app = app
        console.log("making window with",info)
        this.app_id = info.app_id
        this.window_id = info.window_id
        this.window_type = info.window_type
        this.bounds = info.bounds
        this.listeners = {}
    }

    draw_rect(rect, color) {
        console.log('sending draw rect',rect,color)
        this.app.send({DrawRectCommand: {
                    app_id:this.app_id,
                    window_id:this.window_id,
                    rect: rect,
                    color: color,
                }})
    }

    on(type, cb) {
        if(!this.listeners[type]) this.listeners[type] = []
        this.listeners[type].push(cb)
    }
    dispatch(obj) {
        console.log("window got event",obj)
    }
}

class Rect {
    constructor(x,y,w,h) {
        this.x = x
        this.y = y
        this.w = w
        this.h = h
    }
}

class App {
    constructor() {
        this.client = new Socket()
        this.windows = new Map()
    }
    async connect() {
        return new Promise((res,rej)=>{
            this.client.connect(3333,'127.0.0.1',(a,b) => {
                console.log('connected event',a,b)
                res()
                // //on connect, send app connect
                // let msg_out = { AppConnect: { HelloApp: {}}}
                // client.write(`{ "AppConnect": { "HelloApp":{}} }`)
                // // client.write(JSON.stringify(msg_out))
            })
            this.client.on('data',(data)=>{
                let str = data.toString()
                console.log("raw incoming data",str)
                let msg = JSON.parse(str)
                if(msg.MouseDown) return this.windows.get(msg.MouseDown.window_id).dispatch(msg)
                if(msg.KeyDown) return this.windows.get(msg.KeyDown.window_id).dispatch(msg)
                console.log("msg is",msg)
                if(this.cb) this.cb(msg)
            })
        })
    }

    send(obj) {
        this.client.write(JSON.stringify(obj))
    }
    async send_and_wait(obj) {
        console.log("sending",obj)
        let prom = new Promise((res,rej)=>{
            console.log('waiting')
            this.cb = (msg) => {
                console.log("callback completed",msg)
                this.cb = null
                res(msg)
            }
        })
        this.client.write(JSON.stringify(obj))
        return prom
    }

    async open_window(rect) {
        let opened_window = await this.send_and_wait({ OpenWindowCommand:{
                window_type:"plain",
                bounds: rect,
            }})
        let win = new Window(this,opened_window.OpenWindowResponse)
        this.windows.set(win.window_id,win)
        return win
    }
}

function log(...args) {
    console.log(...args)
}

const WHITE = {r:255, g:255, b:255, a:255}
const RED = {r:255, g:0, b:0, a:255}

async function doit() {
    let app = new App()
    await app.connect()
    log("connected")
    let  app_connected = await app.send_and_wait({AppConnect:{HelloApp:{}}})
    console.log("back in msg",app_connected)
    let win = await app.open_window(new Rect(50,50,300,300))
    console.log("done with open window",win.bounds)
    win.draw_rect(new Rect(0,0,300,300),WHITE)
    win.draw_rect(new Rect(50,50,50,100),RED)

    win.on('click',(e) => {
        console.log("got a mouse event",e)
    })
}

doit().then(()=>console.log("fully done")).catch((e)=>console.error(e))
