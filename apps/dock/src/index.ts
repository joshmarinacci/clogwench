import {
    ActionButton, COMMAND_ACTION, LayerView, Rect,
    VBox,
    Label, TextLine,
} from "thneed-gfx";
import {App} from "../../common/src/app";
import {ClogwenchWindowSurface} from "../../common/src/surface";


function start(surface: ClogwenchWindowSurface) {
    let button = new ActionButton()
    button.set_caption("quit")
    button.on(COMMAND_ACTION, async () => {
        process.exit(0)
    })

    let vbox = new VBox()
    vbox.add(button)
    let label = new Label()
    label.set_caption('text label')
    vbox.add(label)

    let button2 = new ActionButton()
    button2.set_caption('action button')
    vbox.add(button2)

    let text = new TextLine()
    text.set_text('some text to edit')
    vbox.add(text)

    let root = new LayerView('root-layer')
    root.add(vbox)
    surface.set_root(root)
    surface.start_input()
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
