import {Socket} from "net";
import {Rect} from "thneed-gfx";

const STD_PORT = 3333

export class App {
    private client: Socket
    private windows: Map<any, any>;

    constructor() {
        this.client = new Socket()
        this.windows = new Map()
    }

    async connect() {
        return new Promise<void>((res, rej) => {
            this.client.connect(STD_PORT, '127.0.0.1', (): void => {
                console.log('connected event')
                res()
            })
            this.client.on('data', (data: Buffer) => {
                let str = data.toString()
                console.log("raw incoming data", str)
                let msg = JSON.parse(str)
                if (msg.MouseDown) return this.windows.get(msg.MouseDown.window_id).dispatch(msg)
                if (msg.MouseUp) return this.windows.get(msg.MouseUp.window_id).dispatch(msg)
                if (msg.KeyDown) return this.windows.get(msg.KeyDown.window_id).dispatch(msg)
                console.log("msg is", msg)
                if (this.cb) this.cb(msg)
            })
        })
    }

    send(obj) {
        this.client.write(JSON.stringify(obj))
    }

    async send_and_wait(obj) {
        console.log("sending", obj)
        let prom = new Promise((res, rej) => {
            console.log('waiting')
            this.cb = (msg) => {
                console.log("callback completed", msg)
                this.cb = null
                res(msg)
            }
        })
        this.client.write(JSON.stringify(obj))
        return prom
    }

    async open_window(rect) {
        let opened_window = await this.send_and_wait({
            OpenWindowCommand: {
                window_type: "plain",
                bounds: rect,
            }
        })
        let win = new Window(this, opened_window.OpenWindowResponse)
        this.windows.set(win.window_id, win)
        return win
    }

    disconnect() {
        console.log("disconnecting the app")
        this.client.end(() => {
            console.log("done ending")
            process.exit(0)
        })
    }
}

export class Window {
    window_id: string;
    bounds: Rect;
    private app_id: string;
    private window_type: string;
    private app: App;
    private listeners: Map<string, any>;

    constructor(app, info) {
        this.app = app
        this.app_id = info.app_id
        this.window_id = info.window_id
        this.window_type = info.window_type
        this.bounds = info.bounds
        this.listeners = new Map<string, any>
    }


    draw_rect(rect, color) {
        // console.log("sending",rect,color)
        this.app.send({
            DrawRectCommand: {
                app_id: this.app_id,
                window_id: this.window_id,
                rect: rect,
                color: color,
            }
        })
    }

    on(type, cb) {
        if (!this.listeners[type]) this.listeners[type] = []
        this.listeners[type].push(cb)
    }

    dispatch(obj) {
        console.log("window got event", obj)
        if (obj.MouseDown) this.fire('mousedown', obj.MouseDown)
        if (obj.MouseUp) this.fire('mouseup', obj.MouseUp)
    }

    fire(type, obj) {
        // console.log("firing", type)
        if (!this.listeners[type]) this.listeners[type] = []
        this.listeners[type].forEach(cb => cb(obj))
    }

    close() {
        // console.log("closing the window")
        // this.app.send_and_wait({WindowCloseRequest:{}})
        return Promise.resolve()
    }
}