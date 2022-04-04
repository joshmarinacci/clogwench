use std::any::{Any, TypeId};
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::iter;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::Iter;
use log::info;
use common::{ARGBColor, Point, Rect, Size};
use crate::core::{ActionEvent, DrawingSurface, EventType, JEventDispatcher, PointerEvent, rect_from_view, UIView};
use crate::UIChild;

pub const HBOX_FILL:ARGBColor = ARGBColor { r: 200, g: 0, b: 200, a: 255 };
pub const VBOX_FILL:ARGBColor = ARGBColor { r: 200, g: 200, b: 200, a: 255 };
pub const BOX_PADDING:i32 = 4;
pub const BUTTON_FILL:ARGBColor = ARGBColor { r: 255, g: 255, b: 255, a: 255};
pub const BUTTON_FILL_ACTIVE:ARGBColor = ARGBColor { r: 0, g: 255, b: 0, a: 255};
pub const BUTTON_TEXT_FILL:ARGBColor = ARGBColor { r: 0, g: 0, b: 0, a: 255};
pub const BUTTON_PADDING:i32 = 4;

pub struct BaseUIView {
    _id:String,
    _name:String,
    _size:Size,
    _position:Point,
    _visible:bool,
}
impl UIView for BaseUIView {
    fn name(&self) -> &str {
        &self._name
    }
    fn size(&self) -> Size {
        self._size
    }
    fn position(&self) -> Point {
        self._position
    }
    fn set_position(&mut self, point: &Point) {
        self._position.copy_from(point)
    }
    // fn visible(&self) -> bool { self._visible }
    fn children(&self) -> Iter<Rc<RefCell<dyn UIView>>> {
        todo!()
    }
    fn layout(&mut self, g: &DrawingSurface, available: &Size) -> Size {
        todo!()
    }
    fn draw(&self, g: &DrawingSurface) {
        todo!()
    }
    fn input(&mut self, e: &PointerEvent) {
        todo!()
    }

    fn hflex(&self) -> bool {
        false
    }
    fn vflex(&self) -> bool {
        false
    }
}

/*
composition based UI

hbox, hspacer, button, etc: are just functions which produce UIViews with specific configurations.
They can use internally whatever they need. They can compose a new View by combining some
existing structs. Maybe start with a BasicUIView which has handlers for all of the common storage. name, visible, size, position.

Button is a rectangle view and a text view and a mouse click responder combined together

HBox is a rectangle view, child storage, and a custom layout function combined together

Label is a text view without anything else

 */

pub struct HBox {
    _id:String,
    _name:String,
    _size:Size,
    _position:Point,
    _children:Vec<UIChild>
}

impl HBox {
    pub fn add(&mut self, child: UIChild) {
        self._children.push(child)
    }
    pub fn make() -> HBox {
        HBox {
            _id: "hbox-id".to_string(),
            _name: "hbox-name".to_string(),
            _size: Size::init(200,20),
            _position: Point::init(0,0),
            _children: vec![]
        }
    }
}

impl UIView for HBox {
    fn name(&self) -> &str {
        return &self._name;
    }
    fn size(&self) -> Size {
        self._size
    }
    fn position(&self) -> Point {
        self._position
    }
    fn set_position(&mut self, point: &Point) {
        self._position.copy_from(point)
    }
    fn children(&self) -> Iter<UIChild> {
        return self._children.iter();
    }
    fn layout(&mut self, g: &DrawingSurface, available: &Size) -> Size {
        // println!("hbox:layout:available {:?}",available);
        //pick a temp size
        self._size = Size::init(available.w-BOX_PADDING-BOX_PADDING,100-BOX_PADDING-BOX_PADDING);

        //find the non-hflex children
        let non_hflex:Vec<&UIChild> = self._children.iter().filter(|ch:&&UIChild|{
            let ch2 = ch.deref().deref().borrow();
            !ch2.hflex()
        }).collect();
        //find the yes-hflex children
        let yes_hflex:Vec<&UIChild> = self._children.iter().filter(|ch:&&UIChild|{
            let ch2 = ch.deref().deref().borrow();
            ch2.hflex()
        }).collect();

        //layout non-hflex children and calc tallest child
        let mut tallest = 0;
        let mut used = 0;
        for ch in non_hflex {
            let size = ch.deref().borrow_mut().layout(g, &self._size);
            used += size.w;
            tallest = i32::max(size.h,tallest)
        }
        //set height to tallest child
        self._size.h = tallest + BOX_PADDING + BOX_PADDING;
        //layout flex children
        if yes_hflex.len() > 0 {
            let av = Size::init((available.w - used - BOX_PADDING - BOX_PADDING) / (yes_hflex.len() as i32), tallest);
            for ch in yes_hflex {
                ch.deref().borrow_mut().layout(g, &av);
            }
        }

        //position children left to right
        let mut x = BOX_PADDING;
        let y = BOX_PADDING;
        for ch in &mut self._children {
            let mut ch2 = ch.deref().deref().borrow_mut();
            ch2.set_position(&Point::init(x, y));
            x += ch2.size().w;
        }
        println!("hbox:layout:final size {:?}",self.size());
        self.size()
    }
    fn draw(&self, g: &DrawingSurface) {
        let bounds = Rect::from_size(self.size());
        g.fill_rect(&bounds,&HBOX_FILL);
    }
    fn input(&mut self, e: &PointerEvent) {
    }

    fn hflex(&self) -> bool {
        false
    }
    fn vflex(&self) -> bool {
        false
    }
}

pub struct VBox {
    _id:String,
    _name:String,
    _size:Size,
    _position:Point,
    _children:Vec<UIChild>
}
impl VBox {
    pub fn add(&mut self, child: UIChild) {
        self._children.push(child)
    }
    pub fn make() -> VBox {
        VBox {
            _id: "vbox-id".to_string(),
            _name: "vbox-name".to_string(),
            _size: Size::init(200,20),
            _position: Point::init(0,0),
            _children: vec![]
        }
    }
}

impl UIView for VBox {
    fn name(&self) -> &str {
        return &self._name;
    }
    fn size(&self) -> Size {
        self._size
    }
    fn position(&self) -> Point {
        self._position
    }
    fn set_position(&mut self, point: &Point) {
        self._position.copy_from(point)
    }
    fn children(&self) -> Iter<UIChild> {
        return self._children.iter();
    }
    fn layout(&mut self, g: &DrawingSurface, available: &Size) -> Size {
        println!("vbox:layout:available {:?}",available);
        //pick a temp size
        self._size = Size::init(available.w-BOX_PADDING-BOX_PADDING,available.h-BOX_PADDING-BOX_PADDING);

        //find the non-hflex children
        let non_vflex:Vec<&UIChild> = self._children.iter().filter(|ch:&&UIChild| !ch.deref().deref().borrow().vflex()).collect();
        //find the yes-hflex children
        let yes_vflex:Vec<&UIChild> = self._children.iter().filter(|ch:&&UIChild|ch.deref().deref().borrow().vflex()).collect();

        //layout non-vflex children and calc tallest child
        let mut widest = 0;
        let mut used = 0;
        for ch in non_vflex {
            let mut ch2 = ch.deref().borrow_mut();
            let size = ch2.layout(g, &self._size);
            used += size.h;
            widest = i32::max(size.w, widest);
            println!("vbox:layout:non-flex-child {} {:?} -> {:?}",ch2.name(), available, size);
        }
        //set height to tallest child
        self._size.w = widest + BOX_PADDING + BOX_PADDING;
        //layout flex children
        // println!("available left {} {}",available.h, used);
        let av = Size::init(available.w-BOX_PADDING-BOX_PADDING,(available.h-used-BOX_PADDING-BOX_PADDING)/(yes_vflex.len() as i32));
        for ch in yes_vflex {
            let mut ch2 = ch.deref().borrow_mut();
            let size = ch2.layout(g, &av);
            println!("vbox:layout:yes-flex-child {} {:?} -> {:?}",ch2.name(), av, size);
        }

        //position children left to right
        let mut x = BOX_PADDING;
        let mut y = BOX_PADDING;
        for ch in &mut self._children {
            let mut ch2 = ch.deref().deref().borrow_mut();
            ch2.set_position(&Point::init(x, y));
            y += ch2.size().h;
        }
        println!("vbox:layout:final size {:?}",self.size());
        self.size()
    }
    fn draw(&self, g: &DrawingSurface) {
        let bounds = Rect::from_size(self.size());
        g.fill_rect(&bounds,&VBOX_FILL);
    }
    fn input(&mut self, e: &PointerEvent) {
    }

    fn hflex(&self) -> bool {
        false
    }
    fn vflex(&self) -> bool {
        false
    }
}

pub struct ActionButton {
    _id:String,
    _name:String,
    _size:Size,
    _position:Point,
    pub(crate) _caption:String,
    _children:Vec<Rc<RefCell<dyn UIView>>>,
    _active:bool,
    pub _dispatcher:JEventDispatcher,
}
impl ActionButton {
    pub(crate) fn make() -> ActionButton {
        ActionButton {
            _id: "action-button-name".to_string(),
            _name: "action-button-name".to_string(),
            _size: Size::init(50, 15),
            _position: Point::init(0,0),
            _caption: "no-caption".to_string(),
            _children: vec![],
            _active:false,
            _dispatcher: JEventDispatcher::new()
        }
    }
}
impl UIView for ActionButton {
    fn name(&self) -> &str {
        &self._name
    }
    fn size(&self) -> Size {
        self._size
    }
    fn position(&self) -> Point {
        self._position
    }
    fn set_position(&mut self, point: &Point) {
        self._position.copy_from(point);
    }
    fn children(&self) -> Iter<UIChild> {
        return self._children.iter();
    }
    fn layout(&mut self, g: &DrawingSurface, available: &Size) -> Size {
        self._size = g.measure_text(&self._caption,"base").grow(BUTTON_PADDING);
        return self.size()
    }
    fn draw(&self, g: &DrawingSurface) {
        let s = self.size();
        let bounds = Rect::from_size(s);
        if self._active {
            g.fill_rect(&bounds, &BUTTON_FILL_ACTIVE);
        } else {
            g.fill_rect(&bounds, &BUTTON_FILL);
        }
        let p = Point::init(1, 1);
        g.fill_text(&self._caption, "base",&p,&BUTTON_TEXT_FILL)
    }
    fn input(&mut self, e: &PointerEvent) {
        match e.etype {
            EventType::MouseDown => {
                self._active = true;
                self._dispatcher.trigger(&ActionEvent {
                    command: "mouse_button".to_string()
                })
            }
            EventType::MouseMove => {}
            EventType::MouseUp => {
                self._active = false;
            }
            EventType::MouseDrag => {}
        }
        println!("action button got a pointer event");
    }

    fn hflex(&self) -> bool {
        false
    }
    fn vflex(&self) -> bool {
        false
    }
}

pub struct Label {
    _id:String,
    _name:String,
    _size:Size,
    _position:Point,
    pub _caption:String,
    _children:Vec<UIChild>,
}
impl Label {
    pub fn make() -> Label {
        Label {
            _id: "label-name".to_string(),
            _name: "label-name".to_string(),
            _size: Size::init(50, 15),
            _position: Point::init(0,0),
            _caption: "no-caption".to_string(),
            _children: vec![]
        }
    }
}
impl UIView for Label {
    fn name(&self) -> &str {
        &self._name
    }
    fn size(&self) -> Size {
        self._size
    }
    fn position(&self) -> Point {
        self._position
    }
    fn set_position(&mut self, point: &Point) {
        self._position.copy_from(point);
    }
    fn children(&self) -> Iter<UIChild> { self._children.iter() }
    fn layout(&mut self, g: &DrawingSurface, available: &Size) -> Size {
        self._size = g.measure_text(&self._caption,"base").grow(BUTTON_PADDING);
        return self.size()
    }
    fn draw(&self, g: &DrawingSurface) {
        let p = Point::init(1, 1);
        g.fill_text(&self._caption, "base",&p,&BUTTON_TEXT_FILL)
    }
    fn input(&mut self, e: &PointerEvent) {
        //noop
    }
    fn hflex(&self) -> bool {
        false
    }
    fn vflex(&self) -> bool {
        false
    }
}

pub struct HSpacer {
    _id:String,
    _name:String,
    _size:Size,
    _position:Point,
    _children:Vec<UIChild>,
}
impl HSpacer {
    pub fn make() -> HSpacer {
        HSpacer {
            _id: "hspacer-name".to_string(),
            _name: "hspacer-name".to_string(),
            _size: Size::init(10,10),
            _position: Point::init(0,0),
            _children: vec![]
        }
    }
}
impl UIView for HSpacer {
    fn name(&self) -> &str {
        &self._name
    }
    fn size(&self) -> Size {
        self._size
    }
    fn position(&self) -> Point {
        self._position
    }
    fn set_position(&mut self, point: &Point) {
        self._position.copy_from(point);
    }
    fn children(&self) -> Iter<UIChild> { self._children.iter() }
    fn layout(&mut self, g: &DrawingSurface, available: &Size) -> Size {
        println!("laying out hspacer. available size is {:?}", available);
        self._size = Size::init(available.w,5);
        return self.size()
    }
    fn draw(&self, g: &DrawingSurface) {
    }
    fn input(&mut self, e: &PointerEvent) {
    }

    fn hflex(&self) -> bool {
        true
    }
    fn vflex(&self) -> bool {
        false
    }
}


pub struct FlexPanel {
    _id:String,
    _name:String,
    _size:Size,
    _position:Point,
    _children:Vec<UIChild>,
    _fill:ARGBColor,
    _hflex: bool,
    _vflex: bool,
}

impl FlexPanel {
    pub(crate) fn make(fill: &ARGBColor, hflex: bool, vflex: bool) -> FlexPanel {
        FlexPanel {
            _id: "flexpanel-name".to_string(),
            _name: "flexpanel-name".to_string(),
            _size: Size::init(10,10),
            _position: Point::init(0,0),
            _children: vec![],
            _fill:fill.clone(),
            _hflex:hflex,
            _vflex:vflex,
        }
    }
}

impl UIView for FlexPanel {
    fn name(&self) -> &str {
        &self._name
    }
    fn size(&self) -> Size {
        self._size
    }
    fn position(&self) -> Point {
        self._position
    }
    fn set_position(&mut self, point: &Point) {
        self._position.copy_from(point);
    }
    fn children(&self) -> Iter<UIChild> { self._children.iter() }

    fn layout(&mut self, g: &DrawingSurface, available: &Size) -> Size {
        println!("laying out the flex panel");
        self._size = available.clone();
        return self.size()
    }

    fn draw(&self, g: &DrawingSurface) {
        println!("drawing the flex panel {:?}", self.size());
        let bounds = Rect::from_size(self.size());
        g.fill_rect(&bounds, &self._fill)
    }

    fn input(&mut self, e: &PointerEvent) { }

    fn hflex(&self) -> bool { self._hflex }

    fn vflex(&self) -> bool { self._vflex }
}
