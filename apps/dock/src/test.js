import {Socket} from "net"
import {Rect, Point} from "thneed-gfx/dist/module.js";

class Window {
    constructor(app,info) {
        this.app = app
        this.app_id = info.app_id
        this.window_id = info.window_id
        this.window_type = info.window_type
        this.bounds = info.bounds
        this.listeners = {}
    }

    draw_rect(rect, color) {
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
        // console.log("window got event",obj)
        if(obj.MouseDown) this.fire('mousedown',obj.MouseDown)
    }
    fire(type, obj) {
        // console.log("firing", type)
        if(!this.listeners[type]) this.listeners[type] = []
        this.listeners[type].forEach(cb => cb(obj))
    }
    close() {
        // console.log("closing the window")
        // this.app.send_and_wait({WindowCloseRequest:{}})
        return Promise.resolve()
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

    disconnect() {
        console.log("disconnecting the app")
        this.client.end(()=>{
            console.log("done ending")
            process.exit(0)
        })
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
    let button_bounds = new Rect(50,50,50,100)
    win.draw_rect(button_bounds,RED)

    win.on('mousedown',async (e) => {
        console.log("got a mouse event",e)
        let pt = new Point(e.x,e.y)
        console.log("down on",pt)
        if(button_bounds.contains(pt)) {
            console.log("inside the button")
            await win.close()
            app.disconnect()
        }
    })
}

doit().then(()=>console.log("fully started")).catch((e)=>console.error(e))
