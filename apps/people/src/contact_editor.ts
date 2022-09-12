import {ActionButton, COMMAND_ACTION, Label, TextLine, VBox} from "thneed-gfx";
import {App, DBObj} from "../../common/src/app";

export class ContactEditor extends VBox {
    private first: TextLine;
    private first_label: Label;
    private save_button: ActionButton;
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
        this.save_button.set_caption("save")
        this.add(this.save_button)

        this.save_button.on(COMMAND_ACTION,async (e) => {
            this.log("saving the contact",this.item)
            this.log("first is",this.first.text)
            this.item.data.first = this.first.text
            this.log("final item is",this.item)
            this.app.db_update(this.item)
        })
    }

    set_contact(item:DBObj) {
        this.item = item
        this.first.set_text(item.data.first)
        // this.last.set_caption(item.data.last)
        // this.email.set_caption(item.data.email)
        // this.phone.set_caption( item.data.phone?item.data.phone:"")

    }
}
