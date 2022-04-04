use common::{ARGBColor, BLACK, Point, Size};
use crate::core::{JEvent, JEventDispatcher};

struct LayoutContext {
}
impl LayoutContext {
    fn new() -> LayoutContext {
        LayoutContext {

        }
    }
    pub fn standard_label_padding(&self) -> i32 {
        return 3
    }
    pub fn measure_text(&self, text:&str) -> Size {
        return Size::init(10,10)
    }
}
trait Layout {
    fn layout(&self, layout: &LayoutContext) -> Size;
}
impl Layout for Button {
    fn layout(&self, layout: &LayoutContext) -> Size {
        let size = layout.measure_text(&self._text).grow(layout.standard_label_padding());
        return size;
    }
}



struct PaintContext {
}

impl PaintContext {
    fn new() -> PaintContext {
        PaintContext {
        }
    }
    pub fn fill_text(&self, text:&str, pos:Point, color:&ARGBColor) {
    }
}
trait Paint {
    fn paint(&self, paint: &PaintContext);
}


struct ClickEvent {
}

impl ClickEvent {
    pub(crate) fn eat(&self) {

    }
}

impl ClickEvent {
    fn new() -> ClickEvent {
        ClickEvent {

        }
    }
}


trait Input {
    fn input(&mut self, event: &mut ClickEvent);
}

struct StandardAction {

}
impl JEvent for StandardAction {

}

struct Button {
    _text:String,
    _events:JEventDispatcher,
}
impl Button {
    fn new(text: &str) -> Button {
        Button {
            _text: text.to_string(),
            _events: JEventDispatcher::new(),
        }
    }
}
impl Paint for Button {
    fn paint(&self, paint: &PaintContext) {
        paint.fill_text(&self._text, Point::init(0,0), &BLACK);
    }
}
impl Input for Button {
    fn input(&mut self, event: &mut ClickEvent) {
        event.eat();
        let act  = StandardAction {
        };
        self._events.trigger(&act)
    }
}

fn test_main() {
    let mut button = Button::new("cool button");
    button._events.add_event_listener(|e:&StandardAction|println!("action happened"));

    let layout_context = LayoutContext::new();
    let size = button.layout(&layout_context);
    let paint_context = PaintContext::new();
    button.paint(&paint_context);

    let mut click = ClickEvent::new();
    button.input(&mut click);
}

/*

open questions:
* can components really reuse implementations?
* how do we split out standard state like name, id, enabled, etc from widget specific state
* how do complex widgets work with rich state. actual data models?
* how does the framework detect at runtime if a View implements a particular trait?
 */