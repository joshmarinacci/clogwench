import {
    ActionButton, LayerView, Rect,
    VBox,
    Label, TextBox, HBox,
} from "thneed-gfx";
import {App, ClogwenchWindowSurface} from "../../common/build";
// import * as child_process from "child_process";
// import {stat} from "fs";


function start(surface: ClogwenchWindowSurface) {
    let vbox = new VBox()
    vbox.set_fill('#00ffdd')
    vbox.set_vflex(true)

    // toolbar
    // text box
    // status bar

    let toolbar = new HBox()
    toolbar.set_name('toolbar')
    toolbar.set_hflex(true)
    toolbar.set_vflex(false)
    let button = new ActionButton()
    button.set_caption('some button')
    toolbar.add(button)
    vbox.add(toolbar)


    let editor = new TextBox()
    editor.set_name('editor')
    vbox.add(editor)

    let statusbar = new HBox()
    statusbar.set_hflex(true)
    statusbar.set_vflex(false)
    let label = new Label()
    label.set_caption("status")
    statusbar.add(label)
    vbox.add(statusbar)



    let root = new LayerView('root-layer')
    root.add(vbox)
    surface.set_root(root)
    surface.start_input()
}

async function doit() {
    let app = new App()
    await app.connect()
    await app.send_and_wait({AppConnect: {HelloApp: {}}})
    let win = await app.open_window(new Rect(200, 50, 300, 250))
    let surface = new ClogwenchWindowSurface(win);
    start(surface)
}

doit().then(() => console.log("fully started")).catch((e) => console.error(e))
