use std::any::{Any, TypeId};
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::slice::Iter;
use log::info;
use common::{APICommand, ARGBColor, BLACK, DrawImageCommand, DrawRectCommand, Point, Rect, Size};
use common::graphics::{GFXBuffer, PixelLayout};
use uuid::Uuid;
use common::client::ClientConnection;
use common::events::{MouseButton, MouseDownEvent};
use common::font::FontInfo2;
use crate::components::ActionButton;
use crate::HBox;

pub trait UIView {
    fn name(&self) -> &str;
    fn size(&self) -> Size;
    fn position(&self) -> Point;
    fn set_position(&mut self, point:&Point);
    fn children(&self) -> Iter<UIChild>;
    fn layout(&mut self, g:&DrawingSurface, available:&Size) -> Size;
    fn draw(&self, g:&DrawingSurface);
    fn input(&mut self, e:&PointerEvent);
}
pub type UIChild = Rc<RefCell<dyn UIView>>;

pub fn rect_from_view(view: &UIChild) -> Rect {
    let p = view.deref().borrow().position();
    let s = view.deref().borrow().size();
    Rect::from_ints(p.x,p.y,s.w,s.h)
}

pub enum EventType {
    MouseDown,
    MouseMove,
    MouseUp,
    MouseDrag,
}
pub enum EventDirection {
    Up,
    Down,
}
pub struct PointerEvent {
    pub button: MouseButton,
    pub etype: EventType,
    pub position: Point,
    pub direction: EventDirection,
}


pub struct DrawingSurface {
    pub appid: Uuid,
    pub winid: Uuid,
    pub client: Rc<RefCell<ClientConnection>>,
    pub font: FontInfo2,
    pub(crate) down: bool,
    _transform:Point,
}

impl DrawingSurface {
    pub(crate) fn init(appid: Uuid, winid: Uuid, font: FontInfo2, client: ClientConnection) -> DrawingSurface {
        DrawingSurface {
            appid,
            winid,
            client:Rc::new(RefCell::new(client)),
            font,
            down: false,
            _transform: Point::init(0,0),
        }
    }
}

pub fn repaint(surf:&mut DrawingSurface, root: UIChild) {
    let size = Size::init(300,150);
    // let mut root = self.root.clone();//.deref().borrow_mut();
    root.deref().borrow_mut().layout(surf, &size);
    surf.draw_view(&root);
}
pub fn start_loop(surf:&mut DrawingSurface, root:UIChild) {
    loop {
        let mut redraw = false;
        let cli = &surf.client;
        for cmd in cli.deref().borrow().rx.try_iter() {
            // info!("got an event {:?}",cmd);
            match cmd {
                APICommand::MouseDown(e) => {
                    surf.down = true;
                    info!("handling mouse down {:?}",e);
                    let position = Point::init(e.x, e.y);
                    info!("mouse position {}",position);
                    let path = surf.scan_path(position,root.clone());
                    let mut evt = PointerEvent {
                        button: e.button,
                        etype: EventType::MouseDown,
                        position: position,
                        direction: EventDirection::Down,
                    };
                    for item in path {
                        let mut view = item.deref().borrow_mut();
                        info!("path item {:?}",view.name());
                        evt.position = evt.position.subtract(&view.position());
                        view.input(&mut evt);
                        redraw = true;
                    }
                }
                APICommand::MouseUp(e) => {
                    if !surf.down {
                        continue;
                    }
                    info!("handling mouse up {:?}",e);
                    let position = Point::init(e.x, e.y);
                    info!("mouse position {}",position);
                    let path = surf.scan_path(position,root.clone());
                    let mut evt = PointerEvent {
                        button: e.button,
                        etype: EventType::MouseUp,
                        position: position,
                        direction: EventDirection::Down,
                    };
                    for item in path {
                        let mut view = item.deref().borrow_mut();
                        info!("path item {:?}",view.name());
                        evt.position = evt.position.subtract(&view.position());
                        view.input(&mut evt);
                        redraw = true;
                    }

                }
                APICommand::SystemShutdown => {
                    info!("CLIENT app:  system is shutting down. bye!");
                    break;
                }
                _ => {}
            }
        }
        if redraw {
            repaint(surf,root.clone());
        }
    }

}

impl DrawingSurface {
    fn handle_mouse_down(&self, e: MouseDownEvent) {
        //             let position = this.surface.screen_to_local(domEvent)
        //             this.last_point = position
        //             this.path = this.scan_path(position)
        //             this.target = this.path[this.path.length-1] // last
        //             let evt = new PointerEvent()
        //             evt.ctx = this.surface
        //             evt.target = this.target
        //             this.propagatePointerEvent(evt,this.path)
        // self.propagate_pointer_event(evt,path);
        //             this.surface.repaint()
        //             domEvent.preventDefault()
    }
    fn scan_path(&self, point: Point, root:UIChild) -> Vec<UIChild> {
        return self.scan_path2(root.clone(),&point);
    }
    fn scan_path2(&self, view:UIChild, point:&Point) -> Vec<UIChild> {
        let mut path:Vec<Rc<RefCell<dyn UIView>>> = vec![];
        let v = view.deref();
        let pos = v.borrow().position();
        let siz  = v.borrow().size();
        let bounds = Rect::from_ints(pos.x,pos.y,siz.w,siz.h);
        //skip if not visible
        //if !root.visible()
        info!("checking {} against {}",point, bounds);
        if siz.w > point.x && siz.h > point.y {
        // if(bounds.contains(point)) {
            info!("inside!");
            for ch in v.borrow().children() {
                let ch2: Rc<RefCell<dyn UIView>> = ch.clone();
                // info!("checking child {}",ch2.deref().borrow().name());
                let pos= ch.deref().borrow().position();
                let mut res = self.scan_path2(ch2, &point.subtract(&pos));
                if res.len() > 0 {
                    // info!("matched child");
                    let mut pth = vec![view.clone()];
                    pth.append(&mut res);
                    return pth;
                }
            }
            return vec![view.clone()];
        }

        return vec![]
    }
    /*
        private calculate_path_to_cursor(view: View, position: Point, path:View[]):boolean {
        // this.log('searching for',position,'on',view.name())
        if(!view) return false
        if (!view.visible()) return false
        let bounds = rect_from_pos_size(view.position(),view.size())
        if (bounds.contains(position)) {
            // @ts-ignore
            if (view.is_parent_view && view.is_parent_view()) {
                let parent = view as unknown as ParentView;
                // go in reverse order to the top drawn children are picked first
                for (let i = parent.get_children().length-1; i >= 0; i--) {
                    let ch = parent.get_children()[i]
                    let pos = position.subtract(view.position())
                    let picked = this.calculate_path_to_cursor(ch,pos,path)
                    if(picked) {
                        path.unshift(ch)
                        return true
                    }
                }
                if(parent.can_receive_mouse()) {
                    return true
                }
            } else {
                return true
            }
        }
        return false
    }

     */

    pub fn measure_text(&self, text: &str, fontname: &str) -> Size {
        return self.font.measure_text(text);
    }
    pub fn fill_text(&self, text: &str, fontname: &str, position: &Point, color: &ARGBColor) {
        let size = self.measure_text(text,fontname);
        let mut text_buffer = GFXBuffer::new(size.w as u32, size.h as u32, &PixelLayout::ARGB());
        text_buffer.clear(&BLACK);
        self.font.draw_text_at(&mut text_buffer,
                               text,
                               0,0,
                               &ARGBColor::new_rgb(0,255,0));
        self.client.deref().borrow().send(APICommand::DrawImageCommand(DrawImageCommand{
            app_id:self.appid,
            window_id:self.winid,
            rect:Rect {
                x:position.x,
                y:position.y+2,
                w: text_buffer.width as i32,
                h: text_buffer.height as i32
            },
            buffer: text_buffer,
        }));

    }
    pub fn fill_rect(&self, bounds: &Rect, color: &ARGBColor) {
        // println!("drawing bounds to a surface {:?} {:?}",bounds,color);
        self.client.deref().borrow().send(APICommand::DrawRectCommand(DrawRectCommand{
            app_id:self.appid,
            window_id:self.winid,
            rect: bounds.clone(),
            color: color.clone(),
        }));
    }
    fn draw_view(&mut self, view: &UIChild) {
        let root = view.deref().borrow_mut();
        self.dotranslate(&root.position());
        root.draw(self);
        for ch in root.children() {
            self.draw_view(ch);
        }
        self.untranslate(&root.position());
    }
    fn dotranslate(&mut self, off: &Point) {
        self._transform = self._transform.add(off);
    }
    fn untranslate(&mut self, off: &Point) {
        self._transform = self._transform.subtract(off);
    }
}


pub struct TypeMap(HashMap<TypeId, Box<dyn Any>>);
impl TypeMap {
    pub fn new() -> TypeMap {
        TypeMap(HashMap::new())
    }
    pub fn set<T:Any + 'static>(&mut self, t: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(t));
    }
    pub fn has<T: 'static+Any>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<T>())
    }
    pub fn get_mut<T: 'static+Any>(&mut self) -> Option<&mut T> {
        self.0.get_mut(&TypeId::of::<T>()).map(|t|{
            t.downcast_mut::<T>().unwrap()
        })
    }
}

pub trait JEvent: 'static {}
//trait JEventListener<E: JEvent> = FnMut(&E) -> () + 'static;

pub struct JEventDispatcher(TypeMap);
type JListenerVec<E> = Vec<Box<dyn FnMut(&E) -> () + 'static>>;

impl JEventDispatcher {
    pub fn new() -> JEventDispatcher {
        JEventDispatcher(TypeMap::new())
    }
    pub fn add_event_listener<E, F>(&mut self, f:F)
        where
            E:JEvent,
            F:FnMut(&E) -> () + 'static
    {
        if !self.0.has::<JListenerVec<E>>() {
            self.0.set::<JListenerVec<E>>(Vec::new());
        }
        let listeners = self.0.get_mut::<JListenerVec<E>>().unwrap();
        listeners.push(Box::new(f));
    }
    pub(crate) fn trigger<E>(&mut self, event: &E)
        where
            E:JEvent
    {
        if let Some(listeners) = self.0.get_mut::<JListenerVec<E>>() {
            for callback in listeners {
                callback(event)
            }
        }
    }
}

pub struct ActionEvent {
    pub(crate) command:String,
}
impl JEvent for ActionEvent {

}
