import {ActionButton, COMMAND_ACTION, LayerView, Rect,}
    from "thneed-gfx";
import {App} from "./app";
import {ClogwenchWindowSurface} from "./surface";

// const WHITE = {r:255, g:255, b:255, a:255}
// const RED = {r:0, g:0, b:255, a:255}

function start(surface: ClogwenchWindowSurface) {
    let button = new ActionButton()
    button.set_caption("a button")
    console.log("action is",COMMAND_ACTION)
    button.on(COMMAND_ACTION, async() => {
        process.exit(0)
    })

    let root = new LayerView('root-layer')
    root.add(button)
    surface.set_root(root)
    surface.start()
    surface.repaint()
}

async function doit() {
    let app = new App()
    await app.connect()
    await app.send_and_wait({AppConnect:{HelloApp:{}}})
    let win = await app.open_window(new Rect(50,50,300,300))
    let surface = new ClogwenchWindowSurface(win);
    start(surface)
}

doit().then(()=>console.log("fully started")).catch((e)=>console.error(e))
