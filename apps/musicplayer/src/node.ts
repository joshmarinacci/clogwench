import {Rect, View} from "thneed-gfx";
import {App} from "../../common/src/app";
import {ClogwenchWindowSurface} from "../../common/src/surface";
import {make_music_player} from "./index";


function start(surface: ClogwenchWindowSurface) {
    let music_root:View = make_music_player(surface) as View;
    surface.set_root(music_root)
    surface.start()
    surface.repaint()
}

async function doit() {
    console.log("making an app")
    let app = new App()
    await app.connect()
    await app.send_and_wait({AppConnect: {HelloApp: {}}})
    let win = await app.open_window(new Rect(50, 50, 600, 300))
    let surface = new ClogwenchWindowSurface(win);
    start(surface)
}

doit().then(() => console.log("fully started")).catch((e) => console.error(e))
