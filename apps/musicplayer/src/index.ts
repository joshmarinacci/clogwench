import {
    ActionButton,
    VBox,
    Label,
    View,
    SurfaceContext,
    HBox,
    SelectList,
    with_props,
    CheckButton,
    RadioButton,
    HSpacer,
    randi,
    TableView, BaseView, Size, ScrollView, COMMAND_ACTION, COMMAND_CHANGE,
} from "thneed-gfx";
import {make_logger} from "josh_js_util"
import {App, DBObj} from "thneed-idealos-common";

const log = make_logger("MusicPlayer")

function make_statusbar() {
    let status_bar = new HBox()
    status_bar.set_vflex(false)
    status_bar.set_hflex(true)
    status_bar.add(new Label("cool status bar"))
    return status_bar
}


class LCDView extends BaseView {
    constructor() {
        super("lcd-view");
        this._name = 'lcd-view'
    }
    draw(g: SurfaceContext): void {
        g.fillBackgroundSize(this.size(),'#cccccc')
        let text = 'LCD View'
        let size = g.measureText(text,'base')
        let x = (this.size().w - size.w)/2
        let y = (this.size().h - size.h)/2
        // g.fillRect(x,y,size.w,size.h,'aqua')
        g.fillStandardText(text,x,y+size.h,'base')
    }

    layout(g: SurfaceContext, available: Size): Size {
        this.set_size(new Size(200,60))
        return this.size()
    }
}

function make_toolbar(player:MusicPlayer) {
    let hbox = new HBox()
    hbox.set_fill('#00ffff')
    hbox.set_hflex(true)
    hbox.set_vflex(false)
    let prev = new ActionButton()
    prev.set_caption('prev')
    hbox.add(prev)
    let play = new ActionButton()
    play.set_caption('play')
    play.on(COMMAND_ACTION, (e) => {
        let track = player.get_selected_track();
        player.play_track(track);
    })
    hbox.add(play)
    let next = new ActionButton()
    next.set_caption('next')
    hbox.add(next)

    hbox.add(new HSpacer())
    hbox.add(new LCDView())
    return hbox
}

export class MusicPlayer extends VBox {
    private song_list: SelectList;
    private _selected_track: DBObj;
    private app: App;

    constructor(app: App) {
        super();
        this.app = app;
        this.set_name('MusicPlayer')
        this.add(make_toolbar(this))


        let middle_layer = new HBox()
        middle_layer.set_vflex(true)
        middle_layer.set_name('middle')
        let source_list = new SelectList(['Library','Playlists','Radio'],(v)=>v)
        source_list.set_name('source-list')

        let scroll = new ScrollView()
        scroll.set_content(source_list)
        scroll.set_pref_width(220)
        scroll.set_vflex(true)
        middle_layer.add(scroll)

        let test_song = {
            id:"some-bad-id",
            data: {
                album:"foo",
                title:"bar",
                artist:"baz"
            }
        }
        let rend = (obj) => {
            return `${obj.data.title} - ${obj.data.artist}`
        }
        this.song_list = new SelectList([test_song],rend)
        this.song_list.on(COMMAND_CHANGE,(e) => {
            this.set_selected_track(e.item)
        })
        middle_layer.add(this.song_list)

        this.add(middle_layer)
        this.add(make_statusbar());
    }
    set_tracks(tracks) {
        // this.log("got the tracks",tracks)
        this.song_list.set_data(tracks)
    }

    get_selected_track():DBObj {
        return this._selected_track;
    }

    play_track(track: DBObj) {
        log.info("music player playing",track);
        this.app.send_and_wait({
            AudioPlayTrackRequest: {
                app_id:this.app.id,
                track:track,
            }
        }).then(r => {
            log.info("got the result",r)
        })
    }

    private set_selected_track(item) {
        this._selected_track = item;
    }
}
export function make_music_player(surface: SurfaceContext, app:App):MusicPlayer {
    let root = new MusicPlayer(app)
    root.set_hflex(true)
    root.set_vflex(true)
    return root
}

