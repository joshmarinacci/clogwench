import {KeyboardInputService, MouseInputService, Point, Rect, Size, Sprite, SurfaceContext, View, ParentView} from "thneed-gfx";
import {Window} from "./app";

export const RED = {r: 0, g: 0, b: 255, a: 255}
const WHITE = {r:255, g:255, b:255, a:255}
const BLACK = {r:0, g:0, b:0, a:255}
const GREEN = {r:0, g:255, b:0, a:255}
const BLUE = {r:255, g:0, b:0, a:255}

export class ClogwenchWindowSurface implements SurfaceContext {
    private win: Window
    private mouse: MouseInputService
    private keyboard: KeyboardInputService
    private _root: View
    private translation: Point;

    constructor(win) {
        this.win = win
        this.translation = new Point(0,0)
        this.mouse = new MouseInputService(this)
        this.keyboard = new KeyboardInputService(this)
        this.win.on('mousedown', async (e) => {
            console.log("got a mouse up event", e)
            let position = new Point(e.x, e.y)
            this.mouse.trigger_mouse_down(position, 0)
        })
        this.win.on('mouseup', async (e) => {
            console.log("got a mouse up event", e)
            let position = new Point(e.x, e.y)
            this.mouse.trigger_mouse_up(position, 0)
        })
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
        this.fillText(caption, new Point(x,y), '#000000')
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
        // console.log("surface starting")
    }

    repaint() {
        // console.log("repainting")
        this.layout_stack();
        this.clear()
        this.draw_stack()
    }

    clear() {
        this.win.draw_rect(new Rect(0, 0, this.win.bounds.w, this.win.bounds.h), WHITE)
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
        // this.log("measuring text:", caption, ',', font_name)
        return new Size(caption.length*12, 12)
    }

    fillBackgroundSize(size:Size, color:string) {
        let c = RED
        if(color.startsWith('#')) c = this.hexstring_to_color(color)
        let rect = new Rect(0,0,size.w,size.h)
        rect.add_position(this.translation)
        this.win.draw_rect(rect,c)
    }

    strokeBackgroundSize(size, color) {
        // this.log('stroking bg ', size, color)
    }

    fillText(caption, pt, color) {
        let c = this.hexstring_to_color(color)
        this.log("filling text",caption,pt,c)
        for(let i=0; i<caption.length; i++) {
            let rect = new Rect(i*12,1,10,10)
            rect.add_position(this.translation)
            this.win.draw_rect(rect,c)
        }
    }


    draw_stack() {
        if (this._root) this.draw_view(this._root)
    }

    draw_view(view) {
        this.log("drawing view", view.name(), view.position(), view.size())
        let pos = view.position()
        if (view.visible()) {
            this.translate(pos)
            view.draw(this);
            this.untranslate(pos)
        }
        if (view.is_parent_view && view.is_parent_view() && view.visible()) {
            let parent = view as unknown as ParentView;
            parent.get_children().forEach(ch => {
                this.draw_view(ch);
            })
        }
    }

    log(...args) {
        console.log(...args)
    }

    private hexstring_to_color(color: string) {
        let r  = Number.parseInt(color.substring(1,3),16)
        let g = Number.parseInt(color.substring(3,5),16)
        let b = Number.parseInt(color.substring(5,7),16)
        // console.log("hex convert",color,r,g,b)
        return { r:r, g:g, b:b, a:255}
    }

    private translate(pos: Point) {
        this.translation = this.translation.add(pos)
    }

    private untranslate(pos: Point) {
        this.translation = this.translation.subtract(pos)
    }
}