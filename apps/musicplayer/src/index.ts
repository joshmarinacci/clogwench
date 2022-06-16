import {
    ActionButton, COMMAND_ACTION, LayerView, Rect,
    VBox,
    Label, TextLine, View, SurfaceContext, HBox, SelectList, with_props, CheckButton, RadioButton, HSpacer,
} from "thneed-gfx";
import {App} from "../../dock/src/app";
import {ClogwenchWindowSurface} from "../../dock/src/surface";


function make_statusbar() {
    let status_bar = new HBox()
    status_bar.set_name('statusbar')
    status_bar.set_fill('#aaa')
    status_bar.set_vflex(false)
    status_bar.set_hflex(true)
    status_bar.add(new Label("cool status bar"))
    status_bar.add(with_props(new CheckButton(), {caption:'Cool?'}))
    status_bar.add(with_props(new RadioButton(), {caption:'Good?'}))
    status_bar.add(with_props(new RadioButton(), {caption:'Better.'}))
    status_bar.add(with_props(new RadioButton(), {caption:'Best!'}))
    status_bar.add(new HSpacer())
    return status_bar
}

function make_music_player(surface: SurfaceContext):View {
    let root = new VBox()
    root.set_name('root')
    // root.add(make_toolbar(surface))

    let middle_layer = new HBox()
    middle_layer.set_vflex(true)
    middle_layer.set_name('middle')
    // let source_list = new SelectList(['Library','Playlists','Radio'],(v)=>v)
    // source_list.set_name('source-list')

    // let scroll = new ScrollView()
    // scroll.set_content(source_list)
    // scroll.set_pref_width(220)
    // scroll.set_vflex(true)
    // middle_layer.add(scroll)
    //
    // middle_layer.add(make_song_list(surface))
    // root.add(middle_layer)
    root.add(make_statusbar());

    root.set_hflex(true)
    root.set_vflex(true)
    return root
}

function start(surface: ClogwenchWindowSurface) {
    let music_root:View = make_music_player(surface) as View;
    surface.set_root(music_root)
    surface.start()
    surface.repaint()
}

async function doit() {
    let app = new App()
    await app.connect()
    await app.send_and_wait({AppConnect: {HelloApp: {}}})
    let win = await app.open_window(new Rect(50, 50, 600, 300))
    let surface = new ClogwenchWindowSurface(win);
    start(surface)
}

doit().then(() => console.log("fully started")).catch((e) => console.error(e))
