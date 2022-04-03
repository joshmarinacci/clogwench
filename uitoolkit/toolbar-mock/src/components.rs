use log::info;
use common::{ARGBColor, Point, Rect, Size};
use crate::core::{DrawingSurface, UIView};

pub const HBOX_FILL:ARGBColor = ARGBColor { r: 200, g: 200, b: 200, a: 255 };
pub const HBOX_PADDING:i32 = 4;
pub const BUTTON_FILL:ARGBColor = ARGBColor { r: 255, g: 255, b: 255, a: 255};
pub const BUTTON_TEXT_FILL:ARGBColor = ARGBColor { r: 0, g: 0, b: 0, a: 255};
pub const BUTTON_PADDING:i32 = 4;

pub struct HBox {
    _id:String,
    _name:String,
    _size:Size,
    _position:Point,
    _children:Vec<ActionButton>
}

impl HBox {
    pub fn add(&mut self, child: ActionButton) {
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
    fn layout(&mut self, g: &impl DrawingSurface, available: &Size) -> Size {
        info!("layout {} avail {:?}",self.name(), available);
        //pick a temp size
        self._size = Size::init(available.w,40);

        //layout children and calc tallest child
        let mut tallest = 0;
        for ch in &mut self._children {
            let size = ch.layout(g, &self._size);
            tallest = i32::max(size.h,tallest)
        }
        //set height to tallest child
        self._size.h = tallest + HBOX_PADDING + HBOX_PADDING;

        //position children left to right
        let mut x = HBOX_PADDING;
        let y = HBOX_PADDING;
        for ch in &mut self._children {
            ch._position.x = x;
            ch._position.y = y;
            x += ch.size().w;
        }
        self.size()
    }
    fn draw(&self, g: &impl DrawingSurface) {
        let bounds = Rect::from_ints(self.position().x,self.position().y,self.size().w,self.size().h);
        info!("draw: {} {}",self.name(), bounds);
        g.fill_rect(&bounds,&HBOX_FILL);
        for ch in &self._children {
            ch.draw(g);
        }
    }
}

pub struct ActionButton {
    _id:String,
    _name:String,
    _size:Size,
    _position:Point,
    pub(crate) _caption:String,
}

impl ActionButton {
    pub(crate) fn make() -> ActionButton {
        ActionButton {
            _id: "action-button-name".to_string(),
            _name: "action-button-name".to_string(),
            _size: Size::init(50, 15),
            _position: Point::init(0,0),
            _caption: "no-caption".to_string()
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

    fn layout(&mut self, g: &impl DrawingSurface, available: &Size) -> Size {
        // info!("layout {}: out button {:?}",self.name(),self._caption);
        self._size = g.measure_text(&self._caption,"base").grow(BUTTON_PADDING);
        // info!("layout {}: size  is now {:?}",self.name(),self.size());
        return self.size()
    }

    fn draw(&self, g: &impl DrawingSurface) {
        // info!("draw, {}, at {:?} {:?}",self.name(),self.position(), self.size());
        let bounds = crate::core::rect_from_view(&self);
        g.fill_rect(&bounds,&BUTTON_FILL);
        let p = self.position().add(Point::init(1,1));
        g.fill_text(&self._caption, "base",&p,&BUTTON_TEXT_FILL)
    }

}
