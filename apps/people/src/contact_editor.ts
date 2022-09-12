import {ActionButton, COMMAND_ACTION, Label, TextLine, VBox} from "thneed-gfx";
import {App, DBObj} from "../../common/src/app";

export class ContactEditor extends VBox {
    private first: TextLine;
    private first_label: Label;
    private save_button: ActionButton;
    private cancel_button: ActionButton;
    private item: DBObj;
    private app: App;

    constructor(app: App) {
        super();
        this.app = app
        this.set_name('contact-editor')

        this.first_label = new Label()
        this.first_label.set_caption("First name")
        this.add(this.first_label)
        this.first = new TextLine()
        this.add(this.first)
        this.first.set_text("-----")

        this.save_button = new ActionButton()
        this.save_button.set_caption("Save")
        this.add(this.save_button)

        this.cancel_button = new ActionButton()
        this.cancel_button.set_caption("Cancel")
        this.add(this.cancel_button)
        this.cancel_button.on(COMMAND_ACTION, (e) => this.hide_editing(e))

        this.save_button.on(COMMAND_ACTION,async (e) => {
            if("id" in this.item) {
                this.save_item(e)
            } else {
                this.add_item(e)
            }
        })
    }

    sync() {
        this.item.data.first = this.first.text
        this.log("final item is",this.item)
    }
    async save_item(e) {
        this.sync()
        let ret = await this.app.db_update(this.item)
        this.hide_editing(e)
        this.fire("DB-CHANGED",{})
    }
    async add_item(e) {
        this.sync()
        let ret = await this.app.db_add(this.item)
        this.hide_editing(e)
        this.fire("DB-CHANGED",{})
    }

    hide_editing(e) {
        this._visible = false
        e.ctx.repaint()
    }

    set_contact(item:DBObj) {
        this.item = item
        this.first.set_text(item.data.first)
        if("id" in this.item) {
            this.save_button.set_caption("Save")
        } else {
            this.save_button.set_caption("Add")
        }
        // this.last.set_caption(item.data.last)
        // this.email.set_caption(item.data.email)
        // this.phone.set_caption( item.data.phone?item.data.phone:"")

    }
}
