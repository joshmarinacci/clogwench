import {
    ActionButton, COMMAND_ACTION, LayerView, Rect,
    VBox,
    Label, TextLine,
} from "thneed-gfx";
import {App} from "../../common/src/app";
import {ClogwenchWindowSurface} from "../../common/src/surface";
import * as child_process from "child_process";


function start(surface: ClogwenchWindowSurface) {
    let vbox = new VBox()

    let label = new Label()
    label.set_caption('Dock')
    vbox.add(label)

    let music_button = new ActionButton()
    music_button.set_caption('Music')
    music_button.on(COMMAND_ACTION, () => {
        console.log("launching the music app")
        child_process.spawn('npm',['run','dev-node'],{
            cwd:'../musicplayer/',
            detached:true,
        })
    })
    vbox.add(music_button)

    let clock_button = new ActionButton()
    clock_button.set_caption('Clock')
    clock_button.on(COMMAND_ACTION, () => {
        console.log("launching the clock app")
        child_process.spawn('cargo',['run'],{
            cwd:'../digital-clock/',
            detached:true,
        })
    })
    vbox.add(clock_button)


    let quit_button = new ActionButton()
    quit_button.set_caption("quit")
    quit_button.on(COMMAND_ACTION, async () => {
        process.exit(0)
    })
    vbox.add(quit_button)


    let root = new LayerView('root-layer')
    root.add(vbox)
    surface.set_root(root)
    surface.start_input()
}

async function doit() {
    let app = new App()
    await app.connect()
    await app.send_and_wait({AppConnect: {HelloApp: {}}})
    let win = await app.open_window(new Rect(50, 50, 100, 300))
    let surface = new ClogwenchWindowSurface(win);
    start(surface)
}

doit().then(() => console.log("fully started")).catch((e) => console.error(e))
