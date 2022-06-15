import {KeyboardInputService, MouseInputService, Point, Rect, Size, Sprite, SurfaceContext, View} from "thneed-gfx";
import {Window} from "./app";

export const RED = {r: 0, g: 0, b: 255, a: 255}

export class ClogwenchWindowSurface implements SurfaceContext {
    private win: Window
    private mouse: MouseInputService
    private keyboard: KeyboardInputService
    private _root: View

    constructor(win) {
        this.win = win
        this.win.on('mousedown', async (e) => {
            console.log("got a mouse up event", e)
            let position = new Point(e.x, e.y)
            console.log("down on", position)
            // if(button_bounds.contains(pt)) {
            //     console.log("inside the button")
            //     await win.close()
            //     app.disconnect()
            // }
            this.mouse.trigger_mouse_down(position, 0)
        })
        this.win.on('mouseup', async (e) => {
            console.log("got a mouse up event", e)
            let position = new Point(e.x, e.y)
            console.log("up on", position)
            this.mouse.trigger_mouse_up(position, 0)
        })
        this.mouse = new MouseInputService(this)
        this.keyboard = new KeyboardInputService(this)
    }

    size(): Size {
        throw new Error("Method not implemented.");
    }

    fill(rect: Rect, color: string) {
        throw new Error("Method not implemented.");
    }

    stroke(rect: Rect, color: string) {
        throw new Error("Method not implemented.");
    }

    fillStandardText(caption: string, x: number, y: number, font_name?: string, scale?: number) {
        throw new Error("Method not implemented.");
    }

    draw_glyph(codepoint: number, x: number, y: number, font_name: string, fill: string, scale?: number) {
        throw new Error("Method not implemented.");
    }

    set_sprite_scale(scale: number) {
        throw new Error("Method not implemented.");
    }

    set_smooth_sprites(sprite_smoothing: boolean) {
        throw new Error("Method not implemented.");
    }

    draw_sprite(pt: Point, sprite: Sprite) {
        throw new Error("Method not implemented.");
    }

    keyboard_focus(): View {
        throw new Error("Method not implemented.");
    }

    set_keyboard_focus(view: View) {
        throw new Error("Method not implemented.");
    }

    is_keyboard_focus(view: View) {
        throw new Error("Method not implemented.");
    }

    release_keyboard_focus(view: View) {
        throw new Error("Method not implemented.");
    }

    view_to_local(pt: Point, view: View): Point {
        throw new Error("Method not implemented.");
    }

    find_by_name(name: string): View {
        throw new Error("Method not implemented.");
    }

    root() {
        return this._root
    }

    set_root(button) {
        this._root = button
    }

    start() {
        console.log("surface starting")
    }

    repaint() {
        console.log("repainting")
        this.layout_stack();
        this.clear()
        this.draw_stack()
    }

    clear() {

    }

    layout_stack() {
        if (!this._root) {
            console.warn("root is null")
        } else {
            let available_size = new Size(this.win.bounds.w, this.win.bounds.h)
            this.log("layout_stack with size", available_size)
            let size = this._root.layout(this, available_size)
            console.log("canvas, root requested", size)
        }
    }

    // measureText(caption: string, font_name?:string):Size;
    measureText(caption, font_name) {
        this.log("measuring text:", caption, ',', font_name)
        return new Size(10, 10)
    }

    fillBackgroundSize(size, color) {
        this.log("filling bg", size, color)
        this.win.draw_rect(new Rect(0, 0, size.w, size.h), RED)
    }

    strokeBackgroundSize(size, color) {
        this.log('stroking bg ', size, color)
    }

    fillText(caption, pt, color) {
        this.log("filling text")
    }


    draw_stack() {
        if (this._root) this.draw_view(this._root)
    }

    draw_view(view) {
        this.log("drawing view", view)
        // this.ctx.save();
        let pos = view.position()
        this.log("position is", pos)
        // this.ctx.translate(pos.x, pos.y)
        // @ts-ignore
        // console.log("drawing",view.id,view.name())
        if (view.visible()) {
            view.draw(this);
        }
        // @ts-ignore
        if (view.is_parent_view && view.is_parent_view() && view.visible()) {
            let parent = view// as unknown as ParentView;
            // if(parent.clip_children()) {
            //     this.ctx.beginPath()
            //     let size = view.size()
            //     this.ctx.rect(0,0,size.w,size.h);
            //     this.ctx.clip()
            // }
            parent.get_children().forEach(ch => {
                // if (this.debug) {
                //     this.ctx.save();
                // }
                this.draw_view(ch);
                // if (this.debug) {
                //     this.ctx.restore()
                // }
            })
        }
        // let bds = rect_from_pos_size(view.position(),view.size())
        // @ts-ignore
        // this.debug_draw_rect(bds, view.name())
        // this.ctx.restore()

    }


    log(...args) {
        console.log(...args)
    }
}