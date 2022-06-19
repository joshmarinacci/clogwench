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
    TableView, BaseView, Size, ScrollView,
} from "thneed-gfx";


function make_statusbar() {
    let status_bar = new HBox()
    status_bar.set_name('statusbar')
    status_bar.set_fill('#aaaaaa')
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

function make_random_word(min,max) {
    let len = randi(min,max)
    var result           = '';
    var characters       = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.toLowerCase();
    var charactersLength = characters.length;
    for ( let i = 0; i < len; i++ ) {
        result += characters.charAt(Math.floor(Math.random() * charactersLength));
        if(i === 0) {
            result = result.toUpperCase()
        }
    }
    return result;
}
function make_random_words(min,max) {
    let count = randi(min,max)
    let res = ''
    for(let i=0; i<count; i++) {
        res += make_random_word(3,12) + ' '
    }
    return res
}

function make_song_list() {
    let songs = []
    for(let i=0; i<3; i++) {
        songs.push({
            type:'song',
            artist:make_random_words(1,3),
            title: make_random_word(2,8),
            album: make_random_word(5,15),
        })
    }
    let song_list = new TableView(songs, ['artist','title','album'], [200,200,300] );
    song_list.set_name('song-list')
    song_list.set_hflex(true)
    song_list.set_vflex(true)
    return song_list
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

function make_toolbar() {
    let hbox = new HBox()
    hbox.set_fill('#00ffff')
    hbox.set_hflex(true)
    hbox.set_vflex(false)
    let prev = new ActionButton()
    prev.set_caption('prev')
    hbox.add(prev)
    let play = new ActionButton()
    play.set_caption('play')
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
    constructor() {
        super();
        this.set_name('MusicPlayer')
        this.add(make_toolbar())


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
            console.log("song is",obj)
            return `${obj.data.title} - ${obj.data.artist}`
        }
        this.song_list = new SelectList([test_song],rend)
        middle_layer.add(this.song_list)

        this.add(middle_layer)
        this.add(make_statusbar());
    }
    set_tracks(tracks) {
        this.log("got the tracks",tracks)
        this.song_list.set_data(tracks)
    }
}
export function make_music_player(surface: SurfaceContext):MusicPlayer {
    let root = new MusicPlayer()
    root.set_hflex(true)
    root.set_vflex(true)
    return root
}

