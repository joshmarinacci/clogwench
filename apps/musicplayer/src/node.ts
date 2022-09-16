import {Rect, View} from "thneed-gfx";
import {App,ClogwenchWindowSurface} from "thneed-idealos-common"
import {make_music_player} from "./index.js";


function start(app: App, surface: ClogwenchWindowSurface) {
    let music_root = make_music_player(surface);
    surface.set_root(music_root)
    surface.start_input()
    surface.repaint()

    setTimeout(async ()=>{
        // console.log('fetching a database query')
        try {
            let tracks = await app.db_query([{kind:'equals',key:'type', value:'song-track'}])
            music_root.set_tracks(tracks)
            surface.repaint()
        } catch (e) {
            console.error(e)
        }
    },3000)
}

async function doit() {
    console.log("making an app")
    let app = new App()
    await app.connect()
    await app.send_and_wait({AppConnect: {HelloApp: {}}})
    let win = await app.open_window(new Rect(50, 50, 600, 300))
    let surface = new ClogwenchWindowSurface(win);
    start(app,surface)
    // app.on_close_window(() => {
    //     console.log("window closed. quitting")
    //     process.exit(0)
    // })
}

doit().then(() => console.log("fully started")).catch((e) => console.error(e))
