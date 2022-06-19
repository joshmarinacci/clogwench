import {Rect, View} from "thneed-gfx";
import {App} from "../../common/src/app";
import {ClogwenchWindowSurface} from "../../common/src/surface";
import {make_music_player} from "./index";


function start(app: App, surface: ClogwenchWindowSurface) {
    let music_root = make_music_player(surface);
    surface.set_root(music_root)
    surface.start()
    surface.repaint()

    setTimeout(async ()=>{
        // console.log('fetching a database query')
        let tracks = await app.db_query({type:'song-track'})
        music_root.set_tracks(tracks)
        surface.repaint()

        let people = await app.db_query({type:"person-contact"})
        console.log("all people are",people)
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
}

doit().then(() => console.log("fully started")).catch((e) => console.error(e))
