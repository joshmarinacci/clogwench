import {Socket} from "net";
import {Rect, Size, Callback} from "thneed-gfx";
import {BufferImage} from "./surface";

const STD_PORT = 3333

export class App {
    private client: Socket
    private windows: Map<any, any>;
    public id: string;
    private _on_close_window_cb: Callback;
    private cb:Callback

    constructor() {
        // console.log("Making a socket")
        this.client = new Socket()
        // console.log("made it")
        this.windows = new Map()
    }

    async connect() {
        return new Promise<void>((res, rej) => {
            this.client.connect(STD_PORT, '127.0.0.1', (): void => {
                // console.log('connected event')
                res()
            })
            this.client.on('data', (data: Buffer) => {
                let str = data.toString()
                // console.log("raw incoming data", str)
                try {
                    let msg = JSON.parse(str)
                    if (msg.AppConnectResponse) {
                        this.id = msg.AppConnectResponse.app_id
                        if (this.cb) this.cb(msg)
                        return
                    }
                    if (msg.MouseDown) return this.windows.get(msg.MouseDown.window_id).dispatch(msg)
                    if (msg.MouseUp) return this.windows.get(msg.MouseUp.window_id).dispatch(msg)
                    if (msg.MouseMove) return this.windows.get(msg.MouseMove.window_id).dispatch(msg)
                    if (msg.KeyDown) return this.windows.get(msg.KeyDown.window_id).dispatch(msg)
                    if (msg.WindowResized) return this.windows.get(msg.WindowResized.window_id).dispatch(msg)
                    if (msg.CloseWindowResponse) {
                        if (this._on_close_window_cb) this._on_close_window_cb({})
                        return
                    }
                    console.warn("msg is", msg)
                    if (this.cb) this.cb(msg)
                } catch (e) {
                    console.log("error JSON parsing",e)
                }
            })
        })
    }

    send(obj) {
        let str = JSON.stringify(obj)
        // console.log('sending',str)
        this.client.write(str)
    }

    async send_and_wait(obj) {
        // console.log("sending", obj)
        let prom = new Promise((res, rej) => {
            // console.log('waiting')
            this.cb = (msg) => {
                // console.log("callback completed", msg)
                this.cb = null
                res(msg)
            }
        })
        this.send(obj)
        return prom
    }

    async open_window(rect) {
        let opened_window = await this.send_and_wait({
            OpenWindowCommand: {
                window_type: "plain",
                window_title: "some-window",
                bounds: rect,
            }
        })
        // @ts-ignore
        let win = new Window(this, opened_window.OpenWindowResponse)
        this.windows.set(win.window_id, win)
        return win
    }

    disconnect() {
        console.log("disconnecting the app")
        this.client.end(() => {
            console.log("done ending")
            // process.exit(0)
        })
    }

    async db_query(param: any) {
        let results = await this.send_and_wait({
            DBQueryRequest:{ app_id:this.id,
                query:param}
        })
        // @ts-ignore
        return results.DBQueryResponse.results
    }

    async db_add(item: DBObj) {
        item.id = "unknown-random-id"
        let results = await this.send_and_wait({
            DBAddRequest: {
                app_id:this.id,
                object:item,
            }
        })
        // console.log("got back",results.DBAddResponse)
        // @ts-ignore
        return results.DBAddResponse
    }

    async db_update(item: DBObj) {
        let results = await this.send_and_wait({
            DBUpdateRequest: {
                app_id:this.id,
                object:item,
            }
        })
        // @ts-ignore
        return results.DBUpdateResponse.results
    }

    async db_delete(item: DBObj) {
        let results = await this.send_and_wait({
            DBDeleteRequest: {
                app_id:this.id,
                object:item,
            }
        })
        // @ts-ignore
        return results.DBDeleteResponse.results
    }

    on_close_window(cb:Callback) {
        this._on_close_window_cb = cb
    }
}

function is_rect_valid(rect: Rect) {
    if(Number.isNaN(rect.x)) return false
    return true
}

function floor_rect(rect:Rect):Rect {
    return new Rect(Math.floor(rect.x),Math.floor(rect.y),Math.floor(rect.w),Math.floor(rect.h))
}
export class Window {
    window_id: string;
    bounds: Rect;
    private app_id: string;
    private window_type: string;
    private app: App;
    private listeners: Map<string, Callback>;
    private buffer: BufferImage;
    private buffered: boolean;

    constructor(app, info) {
        this.app = app
        this.app_id = info.app_id
        this.window_id = info.window_id
        this.window_type = info.window_type
        this.bounds = info.bounds
        this.listeners = new Map<string, Callback>();
        this.buffer = new BufferImage(this.bounds.w,this.bounds.h)
        this.buffered = false
    }


    draw_rect(rect:Rect, color) {
        if(this.buffered) {
            this.buffer.draw_rect(rect, color)
        } else {
            // console.log("window.draw_rect", rect, color)
            this.app.send({
                DrawRectCommand: {
                    app_id: this.app_id,
                    window_id: this.window_id,
                    rect: floor_rect(rect),
                    color: color,
                }
            })
        }
    }
    draw_image(rect:Rect, img:BufferImage):void {
        // console.log("window.draw_image",rect)
        if(!is_rect_valid(rect)) {
            console.error("invalid rect. cannot send",rect)
            throw new Error("invalid rect. cannot send")
        }
        if(this.buffered) {
            this.buffer.draw_image(rect, img);
        } else {
            this.app.send({
                DrawImageCommand: {
                    app_id: this.app_id,
                    window_id: this.window_id,
                    rect: floor_rect(rect),
                    buffer: {
                        layout: {"ARGB": []},
                        id: "31586440-53ac-4a47-83dd-54c88e857fa5",
                        width: img.width,
                        height: img.height,
                        data: img.buffer_data,
                    },
                }
            })
        }
    }

    on(type:string, cb:Callback) {
        if (!this.listeners[type]) this.listeners[type] = []
        this.listeners[type].push(cb)
    }

    dispatch(obj) {
        // console.log("window got event", obj)
        if (obj.MouseDown) this.fire('mousedown', obj.MouseDown)
        if (obj.MouseMove) this.fire('mousemove', obj.MouseUp)
        if (obj.MouseUp) this.fire('mouseup', obj.MouseUp)
        if (obj.KeyDown) this.fire('keydown',obj.KeyDown)
        if (obj.WindowResized) this.set_size(obj.WindowResized.size)
    }

    fire(type:string, obj) {
        // console.log("firing", type)
        if (!this.listeners[type]) this.listeners[type] = []
        this.listeners[type].forEach(cb => cb(obj))
    }

    close() {
        // console.log("closing the window")
        // this.app.send_and_wait({WindowCloseRequest:{}})
        return Promise.resolve()
    }

    flush() {
        if(this.buffered) {
            console.log("sending to the server")
            this.app.send({
                DrawImageCommand: {
                    app_id: this.app_id,
                    window_id: this.window_id,
                    rect: new Rect(0, 0, this.buffer.width, this.buffer.height),
                    buffer: {
                        layout: {"ARGB": []},
                        id: "31586440-53ac-4a47-83dd-54c88e857fa5",
                        width: this.buffer.width,
                        height: this.buffer.height,
                        data: this.buffer.buffer_data,
                    },
                }
            })
        }
    }

    private set_size(size: Size) {
        this.bounds.w = size.w
        this.bounds.h = size.h
        this.buffer = new BufferImage(size.w,size.h)
        this.fire('resize',this)
    }
}

export type DBObj = {
    id:string,
    deleted:boolean,
    data: any
}
