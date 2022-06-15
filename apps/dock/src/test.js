import {Socket} from "net"
import {Rect, Point, COMMAND_ACTION,
    LayerView,
    ActionButton, Size, MouseInputService} from "thneed-gfx";

class ClogwenchWindowSurface {
    constructor(win) {
        this.win = win
        this.win.on('mousedown',async (e) => {
            console.log("got a mouse up event",e)
            let position = new Point(e.x,e.y)
            console.log("down on",position)
            // if(button_bounds.contains(pt)) {
            //     console.log("inside the button")
            //     await win.close()
            //     app.disconnect()
            // }
            this.mouse.trigger_mouse_down(position, 0)
        })
        this.win.on('mouseup',async (e) => {
            console.log("got a mouse up event",e)
            let position = new Point(e.x,e.y)
            console.log("up on",position)
            this.mouse.trigger_mouse_up(position, 0)
        })
        this.mouse = new MouseInputService(this)
    }
    root() {
        return this._root
    }

    set_root(button) {
        this._root = button
    }
    start() {
        console.log("surface starting")
    }
    repaint() {
        console.log("repainting")
        this.layout_stack();
        this.clear()
        this.draw_stack()
    }
    clear() {

    }

    layout_stack() {
        if(!this._root) {
            console.warn("root is null")
        } else {
            let available_size = new Size(this.win.bounds.w,this.win.bounds.h)
            this.log("layout_stack with size",available_size)
            let size = this._root.layout(this, available_size)
            console.log("canvas, root requested",size)
        }
    }

    // measureText(caption: string, font_name?:string):Size;
    measureText(caption, font_name) {
        this.log("measuring text:",caption, ',',font_name)
        return new Size(10,10)
    }
    fillBackgroundSize(size, color) {
        this.log("filling bg",size,color)
        this.win.draw_rect(new Rect(0,0,size.w,size.h),RED)
    }
    strokeBackgroundSize(size, color) {
        this.log('stroking bg ',size,color)
    }
    fillText(caption, pt, color) {
        this.log("filling text")
    }


    draw_stack() {
        if(this._root) this.draw_view(this._root)
    }

    draw_view(view) {
        this.log("drawing view",view)
        // this.ctx.save();
        let pos = view.position()
        this.log("position is",pos)
        // this.ctx.translate(pos.x, pos.y)
        // @ts-ignore
        // console.log("drawing",view.id,view.name())
        if(view.visible()) {
            view.draw(this);
        }
        // @ts-ignore
        if (view.is_parent_view && view.is_parent_view() && view.visible()) {
            let parent = view// as unknown as ParentView;
            // if(parent.clip_children()) {
            //     this.ctx.beginPath()
            //     let size = view.size()
            //     this.ctx.rect(0,0,size.w,size.h);
            //     this.ctx.clip()
            // }
            parent.get_children().forEach(ch => {
                // if (this.debug) {
                //     this.ctx.save();
                // }
                this.draw_view(ch);
                // if (this.debug) {
                //     this.ctx.restore()
                // }
            })
        }
        // let bds = rect_from_pos_size(view.position(),view.size())
        // @ts-ignore
        // this.debug_draw_rect(bds, view.name())
        // this.ctx.restore()

    }



    log(...args) {
        console.log(...args)
    }
}


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
        console.log("window got event",obj)
        if(obj.MouseDown) this.fire('mousedown',obj.MouseDown)
        if(obj.MouseUp) this.fire('mouseup',obj.MouseUp)
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
                if(msg.MouseUp) return this.windows.get(msg.MouseUp.window_id).dispatch(msg)
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
const RED = {r:0, g:0, b:255, a:255}

async function doit() {
    let app = new App()
    await app.connect()
    log("connected")
    let  app_connected = await app.send_and_wait({AppConnect:{HelloApp:{}}})
    console.log("back in msg",app_connected)
    let win = await app.open_window(new Rect(50,50,300,300))
    console.log("done with open window",win.bounds)
    // win.draw_rect(new Rect(0,0,300,300),WHITE)
    // let button_bounds = new Rect(50,50,50,100)
    // win.draw_rect(button_bounds,RED)


    let surface = new ClogwenchWindowSurface(win);
    let button = new ActionButton()
    button.set_caption("a button")
    console.log("action is",COMMAND_ACTION)
    button.on(COMMAND_ACTION, async() => {
        console.log("button action happened")
        await win.close()
        app.disconnect()
    })

    let root = new LayerView('root-layer')
    root.add(button)
    surface.set_root(root)
    surface.start()
    surface.repaint()
    /*
    win.on('mousedown',async (e) => {
        console.log("got a mouse event",e)
        let pt = new Point(e.x,e.y)
        console.log("down on",pt)
        if(button_bounds.contains(pt)) {
            console.log("inside the button")
            await win.close()
            app.disconnect()
        }
    })*/
}

doit().then(()=>console.log("fully started")).catch((e)=>console.error(e))
