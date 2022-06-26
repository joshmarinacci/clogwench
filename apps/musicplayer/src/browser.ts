import {
    TableView, BaseView, Size, CanvasSurface, View,
} from "thneed-gfx";
import {make_music_player} from "./index";

export function start_browser() {
    console.log("starting browser")
    let surface = new CanvasSurface(600,300, 1.0);
    let music_root:View = make_music_player(surface) as View;
    surface.set_root(music_root)
    surface.start_input()
    surface.repaint()
}

