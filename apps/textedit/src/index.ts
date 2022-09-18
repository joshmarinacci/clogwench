import {
    ActionButton, LayerView, Rect,
    VBox,
    Label, TextBox, HBox, COMMAND_ACTION, COMMAND_CHANGE, TextLine, SelectList,
} from "thneed-gfx";
import {App, ClogwenchWindowSurface, DBObj} from "thneed-idealos-common"
// import * as child_process from "child_process";
// import {stat} from "fs";


const TEST_DOCUMENT:DBObj = {
    id:"text-document",
    "deleted":false,
    "data":{
        "type":"text-document",
        "title":"Dummy Text Doc",
        "content":"some text content here"
    }
}
function make_document_list() {
    let data = [TEST_DOCUMENT]
    let list = new SelectList(data,(item)=>{
        return `${item.data.title}`
    })
    list.set_data(data)
    return list
}

function start(surface: ClogwenchWindowSurface, app: App) {
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
    vbox.add(toolbar)

    let search_line = new TextLine()
    search_line.set_text("")
    toolbar.add(search_line)
    search_line.set_pref_width(150)
    search_line.on(COMMAND_ACTION,async ()=>{
        // console.log("enter in the search",search_line.text)
        let query = search_line.text
        let results:DBObj[] = await app.db_query([
            {kind:'equals',key:'type',value:'text-document'},
            {kind:'substringi',key:'first',value:query},
        ]);
        list.set_data(results)
        surface.repaint()
    })
    let save_button = new ActionButton()
    save_button.set_caption('save')
    toolbar.add(save_button)


    let middle = new HBox()
    vbox.add(middle)

    let list = make_document_list()
    middle.add(list)

    let editor = new TextBox()
    editor.set_name('editor')
    middle.add(editor)


    let selected_document:DBObj = TEST_DOCUMENT
    editor.set_text(selected_document.data.content)
    list.on(COMMAND_CHANGE,(e)=>{
        selected_document = e.item
        // editor.set_text(selected_document.data.content)
        // let contact_view = make_contact_view()
        // current_view.set_content(contact_view)
        // contact_view.set_contact(e.item)
    })

    const refresh_list = async () => {
        let results = await app.db_query(
            [{
                kind:'equals',
                key:'type',
                value:'text-document',
            }]
        )
        list.set_data(results)
        surface.repaint()
    }
    save_button.on(COMMAND_ACTION, async() => {

    })

    let root = new LayerView('root-layer')
    root.add(vbox)
    surface.set_root(root)
    surface.start_input()
    setTimeout(async () => {
        refresh_list()
    },500)
}

async function doit() {
    let app = new App()
    await app.connect()
    await app.send_and_wait({AppConnect: {HelloApp: {}}})
    let win = await app.open_window(new Rect(200, 50, 300, 250))
    let surface = new ClogwenchWindowSurface(win);
    start(surface, app)
}

doit().then(() => console.log("fully started")).catch((e) => console.error(e))
