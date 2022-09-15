import {
    KeyboardInputService,
    MouseInputService,
    Point,
    Rect,
    Size,
    Sprite,
    SurfaceContext,
    View,
    ParentView,
    Modifiers,
    Callback,
    SpriteGlyph,
    StandardTextHeight,
    BASE_FONT,
} from "thneed-gfx";
import {Window} from "./app.js";


import {IDEALOS_KEYBOARD_CODE, IDEALOS_KEYBOARD_KEY} from "./generated.js";

type Color = {
    r:number,
    g:number,
    b:number,
    a:number
}

export const RED:Color = {r: 0, g: 0, b: 255, a: 255}
export const MAGENTA:Color = {r:255, g:0, b:255, a:255}
const WHITE:Color = {r:255, g:255, b:255, a:255}
const BLACK:Color = {r:0, g:0, b:0, a:255}
const GREEN = {r:0, g:255, b:0, a:255}
const BLUE = {r:255, g:0, b:0, a:255}
const TRANSPARENT:Color = {r:255, g:0, b:255, a:0}

export type IdealosKeyEvent = {
    key:string
    code:string
}

export function ideal_os_key_to_thneed_code(inp:IdealosKeyEvent):IdealosKeyEvent {
    // console.log("converting idealos key event",inp)
    let out:IdealosKeyEvent = {
        code: "",
        key: ""
    }
    if(IDEALOS_KEYBOARD_CODE[inp.code]) {
        out.code = IDEALOS_KEYBOARD_CODE[inp.code]
    }
    if(IDEALOS_KEYBOARD_KEY[out.code]) {
        out.key = IDEALOS_KEYBOARD_KEY[out.code]
    }
    if(inp.key && inp.key !== '') {
        out.key = inp.key
    }
    // console.log("to thneed event:",out)
    return out
}

// console.log("surface loaded font",basefont_data)
export class BufferImage {
    width: number;
    height: number;
    buffer_data: number[];

    constructor(w:number, h:number) {
        this.width = w
        this.height = h
        this.buffer_data = []
        for(let i=0; i<this.width*this.height; i++) {
            this.buffer_data[i*4+0] = 255
            this.buffer_data[i*4+1] = 255
            this.buffer_data[i*4+2] = 0
            this.buffer_data[i*4+3] = 255
        }
    }
    set_pixel(x:number, y:number, color:Color) {
        if(x < 0) return
        if(y < 0) return
        if(x >= this.width) return
        if(y >= this.height) return
        let n = (y*this.width+x)
        this.buffer_data[n*4 + 0] = color.a
        this.buffer_data[n*4 + 1] = color.r
        this.buffer_data[n*4 + 2] = color.g
        this.buffer_data[n*4 + 3] = color.b
    }

    draw_rect(rect: Rect, color:Color):void {
        for(let i = rect.x; i<rect.right(); i++) {
            for(let j=rect.y; j<rect.bottom(); j++) {
                // console.log("setting",i,j)
                this.set_pixel(i,j,color)
            }
        }
    }

    draw_image(rect: Rect, img: BufferImage) {
        this.draw_rect(rect,MAGENTA);
    }
}
export class BufferFont {
    private data: any;
    private metas:Map<number,SpriteGlyph>;
    private scale = 1;
    constructor(data) {
        this.data = data
        this.metas = new Map();
        this.data.glyphs.forEach(gl => {
            this.generate_image(gl)
            this.metas.set(gl.meta.codepoint,gl)
        })
    }
    measureText(text) {
        let xoff = 0
        let h = 0
        for(let i=0; i<text.length; i++) {
            let cp = text.codePointAt(i)
            if(this.metas.has(cp)) {
                let glyph = this.metas.get(cp)
                let sw = glyph.w - glyph.meta.left - glyph.meta.right
                xoff += sw + 1
                h = Math.max(h,glyph.h)
            } else {
                xoff += 10
                h = Math.max(h,10)
            }
        }
        return new Size(xoff*this.scale,h*this.scale)
    }

    fillText(win: Window, text: string, x: number, y: number, scale?: number) {
        // this.log("filling text",text)
        if(!scale) scale = 1
        // ctx.fillStyle = 'red'
        let size = this.measureText(text)
        let xoff = 0
        let yoff = 2
        // ctx.fillRect(x+xoff, y+yoff, size.w, size.h)
        for (let i = 0; i < text.length; i++) {
            let cp = text.codePointAt(i)
            let dx = x + xoff*this.scale*scale
            if (this.metas.has(cp)) {
                let glyph = this.metas.get(cp)
                // ctx.imageSmoothingEnabled = false
                //@ts-ignore
                // let img = glyph.img
                // console.log(glyph)
                let sx = glyph.meta.left
                let sy = 0
                let sw = glyph.w - glyph.meta.left - glyph.meta.right
                let sh = glyph.h //- glyph.meta.baseline
                let dy = y + (yoff+glyph.meta.baseline-1)*this.scale*scale
                let dw = sw*this.scale*scale
                let dh = sh*this.scale*scale
                // @ts-ignore
                // console.log("bf: ", glyph.img)
                // win.draw_rect(new Rect(dx,dy,dw,dh),BLACK)
                win.draw_image(new Rect(dx,dy,dw,dh), glyph.img)
                // ctx.drawImage(img, sx,sy,sw,sh, dx,dy, dw,dh)
                xoff += sw + 1
            } else {
                //missing the glyph
                let ew = 8
                let dy = y + (yoff)*this.scale*scale
                win.draw_rect(new Rect(dx,dy,8,8),BLACK)
                // ctx.strokeRect(dx,dy,ew*this.scale*scale,ew*this.scale*scale)
                xoff += ew + 1

            }
        }
    }

    draw_glpyh(win:Window, cp:number, x:number, y:number, scale?:number) {
        if(!scale) scale = 1
        // this.log("draw_glyph",cp)
        let xoff = 0
        let yoff = 2
        if(this.metas.has(cp)) {
            // this.log("have glyph",cp)
            let glyph = this.metas.get(cp)
            // this.log(glyph)
            // this.log(xoff, x, this.scale, scale)
            // ctx.imageSmoothingEnabled = false
            //@ts-ignore
            // let img = glyph.img
            let sx = glyph.meta.left
            let sy = 0
            let sw = glyph.w - glyph.meta.left - glyph.meta.right
            let sh = glyph.h //- glyph.meta.baseline
            let dx = x + xoff*this.scale*scale
            let dy = y + (yoff+glyph.meta.baseline-1)*this.scale*scale
            let dw = sw*this.scale*scale
            let dh = sh*this.scale*scale
            // @ts-ignore
            win.draw_image(new Rect(dx,dy,dw,dh), glyph.img)
            // ctx.drawImage(img, sx,sy,sw,sh, dx,dy, dw,dh)
        } else {
            this.log("missing glyph",cp)
        }
    }

    private generate_image(gl) {
        // this.log("generate image")
        let w = gl.w-gl.meta.left-gl.meta.right
        gl.img = new BufferImage(w,gl.h)
        // c.fillRect(0,0,gl.img.width,gl.img.height)
        for (let j = 0; j < gl.h; j++) {
            for (let i = 0; i < gl.w; i++) {
                let n = j * gl.w + i;
                let v = gl.data[n];
                if(v %2 === 0) {
                    gl.img.set_pixel(i-gl.meta.left,j,TRANSPARENT)
                }
                if(v%2 === 1) {
                    gl.img.set_pixel(i-gl.meta.left,j,BLACK)
                }
            }
        }
    }

    private log(...args) {
        console.log("BufferFont:", ...args)
    }
}
export class ClogwenchWindowSurface implements SurfaceContext {
    private win: Window
    private mouse: MouseInputService
    private keyboard: KeyboardInputService
    private _root: View
    private translation: Point;
    private font: BufferFont;
    private _keyboard_focus: View|null;
    protected _input_callback: Callback;

    constructor(win) {
        this.win = win
        this.translation = new Point(0,0)
        this.mouse = new MouseInputService(this)
        this.keyboard = new KeyboardInputService(this)
        this.win.on('mousedown', (e) => {
            // console.log("got a mouse up event", e)
            let position = new Point(e.x, e.y)
            this.mouse.trigger_mouse_down(position, 0)
        })
        this.win.on('mousemove', (e) => {
            // console.log("got a mouse move event", e)
            let position = new Point(e.x, e.y)
            this.mouse.trigger_mouse_move(position, 0)
        })
        this.win.on('mouseup', (e) => {
            // console.log("got a mouse up event", e)
            let position = new Point(e.x, e.y)
            this.mouse.trigger_mouse_up(position, 0)
        })
        this.win.on('keydown',(evt) => {
            let mod:Modifiers = {
                alt: false, ctrl: false, meta: false, shift: false
            }
            //ArrowRight
            let e = ideal_os_key_to_thneed_code(evt);
            this.keyboard.trigger_key_down(e.key,e.code, mod)
        })
        this.win.on('resize',() => this.repaint())
        let name = 'base'
        let fnt = BASE_FONT.fonts.find(ft => ft.name === name)
        this.font = new BufferFont(fnt)
    }

    size(): Size {
        throw new Error("Method size() not implemented.");
    }

    fill(rect: Rect, color: string) {
        let c = this.hexstring_to_color(color)
        rect = rect.clone()
        rect.add_position(this.translation)
        this.win.draw_rect(rect,c)
    }

    stroke(rect: Rect, color: string) {
        let c = this.hexstring_to_color(color)
        let r2 = rect.clone()
        r2.add_position(this.translation)
        this.win.draw_rect(new Rect(r2.x,r2.y,r2.w,1),c)
        this.win.draw_rect(new Rect(r2.x,r2.y+r2.h-1,r2.w,1),c)
        this.win.draw_rect(new Rect(r2.x,r2.y,1,r2.h),c)
        this.win.draw_rect(new Rect(r2.x+r2.w-1,r2.y,1,r2.h),c)
    }

    fillStandardText(caption: string, x: number, y: number, font_name?: string, scale?: number) {
        this.fillText(caption, new Point(x,y), '#000000')
    }

    draw_glyph(codepoint: number, x: number, y: number, font_name: string, fill: string, scale?: number) {
        let ptx = new Point(x,y)
        let pt = ptx.add(this.translation)
        this.font.draw_glpyh(this.win, codepoint, pt.x, pt.y)
    }

    set_sprite_scale(scale: number) {
        throw new Error("Method set_sprite_scale() not implemented.");
    }

    set_smooth_sprites(sprite_smoothing: boolean) {
        throw new Error("Method set_smooth_sprites() not implemented.");
    }

    draw_sprite(pt: Point, sprite: Sprite) {
        throw new Error("Method draw_sprite() not implemented.");
    }

    keyboard_focus(): View {
        return this._keyboard_focus
    }

    set_keyboard_focus(view: View) {
        this._keyboard_focus = view
    }

    is_keyboard_focus(view: View) {
        return this._keyboard_focus === view && this._keyboard_focus
    }

    release_keyboard_focus(view: View) {
        this._keyboard_focus = null
        // throw new Error("Method release_keyboard_focus() not implemented.");
    }

    view_to_local(pt: Point, view: View): Point {
        throw new Error("Method view_to_local() not implemented.");
    }

    find_by_name(name: string): View {
        throw new Error("Method find_by_name() not implemented.");
    }

    root() {
        return this._root
    }

    set_root(button) {
        this._root = button
    }

    start() {
        // console.log("clogwench surface starting")
    }
    start_input() {
        this._input_callback = () => {
            // console.log("repainting on input")
            this.repaint()
        }
        this.repaint()
    }

    repaint() {
        // console.log("repainting")
        this.layout_stack();
        this.clear()
        this.draw_stack()
        this.win.flush()
        // console.log("flushed")
    }

    clear() {
        this.win.draw_rect(new Rect(0, 0, this.win.bounds.w, this.win.bounds.h), WHITE)
    }

    layout_stack() {
        if (!this._root) {
            console.warn("root is null")
        } else {
            let available_size = new Size(this.win.bounds.w, this.win.bounds.h)
            // this.log("layout_stack with size", available_size)
            let size = this._root.layout(this, available_size)
            // console.log("canvas, root requested", size)
        }
    }

    measureText(caption, font_name) {
        return this.font.measureText(caption)
    }

    fillBackgroundSize(size:Size, color:string) {
        let c = RED
        if(color.startsWith('#')) c = this.hexstring_to_color(color)
        let rect = new Rect(0,0,size.w,size.h)
        rect.add_position(this.translation)
        this.win.draw_rect(rect,c)
    }

    strokeBackgroundSize(size, color) {
        let c = this.hexstring_to_color(color)
        let rect = new Rect(0,0,size.w,size.h)
        rect.add_position(this.translation)
        this.win.draw_rect(new Rect(rect.x,rect.y,rect.w,1),c)
        this.win.draw_rect(new Rect(rect.x,rect.y+rect.h-1,rect.w,1),c)
        this.win.draw_rect(new Rect(rect.x,rect.y,1,rect.h),c)
        this.win.draw_rect(new Rect(rect.x+rect.w-1,rect.y,1,rect.h),c)
    }

    fillText(caption, ptx, color) {
        let c = this.hexstring_to_color(color)
        // this.log("filling text",caption,ptx,c)
        let pt = ptx.add(this.translation)
        this.font.fillText(this.win, caption,pt.x,pt.y-StandardTextHeight)
    }


    draw_stack() {
        if (this._root) this.draw_view(this._root)
    }

    draw_view(view) {
        // this.log("drawing view", view.name(), view.position(), view.size())
        let pos = view.position()
        this.translate(pos)
        if (view.visible()) {
            view.draw(this);
        }
        if (view.is_parent_view && view.is_parent_view() && view.visible()) {
            let parent = view as unknown as ParentView;
            parent.get_children().forEach(ch => {
                this.draw_view(ch);
            })
        }
        this.untranslate(pos)
    }

    log(...args) {
        console.log(...args)
    }

    private hexstring_to_color(color: string):Color {
        if(!color) return MAGENTA
        if(color.length !== 7) {
            console.warn(`bad color ${color}`)
            return MAGENTA
        }
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
