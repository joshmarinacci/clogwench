import {
    ActionButton,
    BaseParentView,
    COMMAND_ACTION,
    COMMAND_CHANGE,
    HBox,
    Label,
    LayerView,
    Rect,
    SelectList,
    Size,
    SurfaceContext,
    TextLine,
    VBox,
    View,
} from "thneed-gfx";
import {App, DBObj} from "../../common/src/app";
import {ClogwenchWindowSurface} from "../../common/src/surface";
import {ContactEditor} from "./contact_editor";

class ContactView extends VBox {
    private first: Label;
    private last: Label;
    private email: Label;
    private phone: Label;
    constructor() {
        super();
        this.set_name('contact-view')
        this.first = new Label()
        this.first.set_caption('nothing')
        this.add(this.first)
        this.last = new Label()
        this.last.set_caption('selected')
        this.add(this.last)
        this.email = new Label()
        this.email.set_caption('')
        this.add(this.email)

        this.phone = new Label()
        this.phone.set_caption('')
        this.add(this.phone)
    }

    set_contact(item) {
        // this.log("item is",item)
        this.first.set_caption(item.data.first)
        this.last.set_caption(item.data.last)
        this.email.set_caption(item.data.email)
        this.phone.set_caption( item.data.phone?item.data.phone:"")
    }
}
function make_contact_view():ContactView {
    let view = new ContactView()
    view.set_hflex(true)
    view.set_vflex(true)
    return view
}

class SwapView extends BaseParentView {
    constructor() {
        super("swap-view")
        this.set_name("swap-view")
    }
    draw(g: SurfaceContext) {
    }
    layout(g: SurfaceContext, available: Size): Size {
        this.set_size(available)
        this._children.forEach(ch => ch.layout(g,available))
        return this.size()
    }
    set_content(view:View) {
        this._children = []
        this.add(view)
    }
    clear() {
        this._children = []
    }
}

const TEST_CONTACT:DBObj = {
    "id": "addr-id-03xxx",
    "deleted":false,
    "data": {
        "type": "person-contact",
        "first": "Billy",
        "last": "Bob",
        "email": "billybob@billybob.com"
    }
}

function make_empty_contact():DBObj {
    return {
        "deleted":false,
        "data": {
            type:"person-contact",
            first:"",
            last:"",
            email:""
        }
    }
}

function make_contacts_list() {
    let data = [TEST_CONTACT]
    let list = new SelectList(data,(item)=>{
        return `${item.data.first} ${item.data.last}`
    })
    list.set_data(data)
    return list
}

function start(surface: ClogwenchWindowSurface, app:App) {
    let vbox = new VBox()
    vbox.set_fill('#00ffdd')
    vbox.set_vflex(true)

    //results list
    //contact view
        // first, last
        // email, phone number
    //add new contact
    //edit existing contact
    //delete existing contact

    let toolbar = new HBox()
    toolbar.set_name('toolbar')
    toolbar.set_hflex(true)
    toolbar.set_vflex(false)
    toolbar.set_fill('#c0c0c0')
    vbox.add(toolbar)

    let search_line = new TextLine()
    search_line.set_text("")
    toolbar.add(search_line)
    search_line.set_pref_width(150)
    search_line.on(COMMAND_ACTION,async ()=>{
        // console.log("enter in the search",search_line.text)
        let query = search_line.text
        let results:DBObj[] = await app.db_query([
            {kind:'equals',key:'type',value:'person-contact'},
            {kind:'substringi',key:'first',value:query},
            ]);
        list.set_data(results)
        surface.repaint()
    })


    let add_button = new ActionButton()
    add_button.set_caption('add')
    toolbar.add(add_button)
    let edit_button = new ActionButton()
    edit_button.set_caption('edit')
    toolbar.add(edit_button)
    let delete_button = new ActionButton()
    delete_button.set_caption('delete')
    toolbar.add(delete_button)

    let middle = new HBox()
    let list = make_contacts_list()
    middle.add(list)
    let current_view = new SwapView()
    middle.add(current_view)
    vbox.add(middle)

    let selected_contact:DBObj = TEST_CONTACT
    current_view.set_content(make_contact_view())
    list.on(COMMAND_CHANGE,(e)=>{
        selected_contact = e.item
        let contact_view = make_contact_view()
        current_view.set_content(contact_view)
        contact_view.set_contact(e.item)
    })

    add_button.on(COMMAND_ACTION, async () => {
        let contact_editor = new ContactEditor(app)
        contact_editor.set_contact(make_empty_contact())
        current_view.set_content(contact_editor)
    })

    edit_button.on(COMMAND_ACTION, async () => {
        let contact_editor = new ContactEditor(app)
        contact_editor.set_contact(selected_contact)
        current_view.set_content(contact_editor)
    })

    let root = new LayerView('root-layer')
    root.add(vbox)
    surface.set_root(root)
    surface.start_input()

    setTimeout(async () => {
        let results = await app.db_query(
            [{
                kind:'equals',
                key:'type',
                value:'person-contact',
            }]
        )
        list.set_data(results)
        surface.repaint()
    },500)

}

async function doit() {
    let app = new App()
    await app.connect()
    await app.send_and_wait({AppConnect: {HelloApp: {}}})
    let win = await app.open_window(new Rect(200, 50, 500, 250))
    let surface = new ClogwenchWindowSurface(win);
    start(surface,app)
}

doit().then(() => console.log("fully started")).catch((e) => console.error(e))
