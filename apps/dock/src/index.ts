import {ActionButton, COMMAND_ACTION, LayerView, Rect,
    VBox,
    Label,
} from "thneed-gfx";
import {App} from "./app";
import {ClogwenchWindowSurface} from "./surface";


function start(surface: ClogwenchWindowSurface) {
    let button = new ActionButton()
    button.set_caption("foobutton")
    button.on(COMMAND_ACTION, async () => {
        process.exit(0)
    })

    let vbox = new VBox()
    vbox.add(button)
    let label = new Label()
    label.set_caption('foolabel')
    vbox.add(label)

    let button2 = new ActionButton()
    button2.set_caption('barbutton')
    vbox.add(button2)

    let root = new LayerView('root-layer')
    root.add(vbox)
    surface.set_root(root)
    surface.start()
    surface.repaint()
}

async function doit() {
    let app = new App()
    await app.connect()
    await app.send_and_wait({AppConnect: {HelloApp: {}}})
    let win = await app.open_window(new Rect(50, 50, 300, 300))
    let surface = new ClogwenchWindowSurface(win);
    start(surface)
}

doit().then(() => console.log("fully started")).catch((e) => console.error(e))
