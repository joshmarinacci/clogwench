use std::sync::mpsc::RecvError;
use log::{info, LevelFilter, set_logger};
use uuid::Uuid;
use common::{APICommand, ARGBColor, BLACK, DrawImageCommand, DrawRectCommand, HelloApp, OpenWindowCommand, Padding, Point, Rect, Size, WHITE};
use common::client::ClientConnection;
use common::font::{FontInfo2, load_font_from_json};
use common::graphics::{GFXBuffer, PixelLayout};
use cool_logger::CoolLogger;

static COOL_LOGGER:CoolLogger = CoolLogger;

pub const HBOX_FILL:ARGBColor = ARGBColor { r: 200, g: 200, b: 200, a: 255 };
pub const HBOX_PADDING:i32 = 4;
pub const BUTTON_FILL:ARGBColor = ARGBColor { r: 255, g: 255, b: 255, a: 255};
pub const BUTTON_TEXT_FILL:ARGBColor = ARGBColor { r: 0, g: 0, b: 0, a: 255};
pub const BUTTON_PADDING:i32 = 4;

pub trait DrawingSurface {
    fn measure_text(&self, text:&str, fontname:&str) -> Size;
    fn fill_text(&self, text:&str, fontname:&str, position:&Point, color:&ARGBColor);
    fn fill_rect(&self, bounds:&Rect, color:&ARGBColor);
}

pub struct DrawingSurfaceImpl {
    pub appid: Uuid,
    pub winid: Uuid,
    pub client: ClientConnection,
    pub font: FontInfo2,
}

impl DrawingSurface for DrawingSurfaceImpl {
    fn measure_text(&self, text: &str, fontname: &str) -> Size {
        return self.font.measure_text(text);
    }

    fn fill_text(&self, text: &str, fontname: &str, position: &Point, color: &ARGBColor) {
        let size = self.measure_text(text,fontname);
        let mut text_buffer = GFXBuffer::new(size.w as u32, size.h as u32, &PixelLayout::ARGB());
        text_buffer.clear(&BLACK);
        self.font.draw_text_at(&mut text_buffer,
                               text,
                               0,0,
                               &ARGBColor::new_rgb(0,255,0));
        self.client.send(APICommand::DrawImageCommand(DrawImageCommand{
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

    fn fill_rect(&self, bounds: &Rect, color: &ARGBColor) {
        // println!("drawing bounds to a surface {:?} {:?}",bounds,color);
        self.client.send(APICommand::DrawRectCommand(DrawRectCommand{
            app_id:self.appid,
            window_id:self.winid,
            rect: bounds.clone(),
            color: color.clone(),
        }));
    }
}

pub trait UIView {
    fn size(&self) -> Size;
    fn name(&self) -> &str;
    fn position(&self) -> Point;
    fn draw(&self, g:&impl DrawingSurface);
    fn layout(&mut self, g:&impl DrawingSurface, available:&Size) -> Size;
}

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
    _caption:String,
}

impl ActionButton {
    fn make() -> ActionButton {
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
        let bounds = rect_from_view(&self);
        g.fill_rect(&bounds,&BUTTON_FILL);
        let p = self.position().add(Point::init(1,1));
        g.fill_text(&self._caption, "base",&p,&BUTTON_TEXT_FILL)
    }

}

fn rect_from_view(view: &&ActionButton) -> Rect {
    Rect::from_ints(view.position().x,view.position().y,view.size().w,view.size().h)
}


fn main() {
    set_logger(&COOL_LOGGER).map(|()|log::set_max_level(LevelFilter::Info));

    info!("toolbar app starting and connecting");
    let bounds = Rect::from_ints(50,50,300,300);
    let mut px = 50;
    let mut py = 50;
    let mut appid = Uuid::new_v4();

    let client = ClientConnection::init().expect("Can't connect to the central");
    //open window and wait
    let resp: Result<APICommand, RecvError> = client.send_and_wait(APICommand::AppConnect(HelloApp{}));
    match resp {
        Ok(APICommand::AppConnectResponse(appinfo)) => {
            appid = appinfo.app_id
        }
        _ => {
            panic!("error. response should have been from the app connect")
        }
    }
    let resp2: Result<APICommand, RecvError> = client.send_and_wait(APICommand::OpenWindowCommand(OpenWindowCommand{
        window_type: String::from("plain"),
        bounds: bounds,
    }));
    let mut winid = Uuid::new_v4();
    match resp2 {
        Ok(APICommand::OpenWindowResponse(wininfo)) => {
            winid = wininfo.window_id
        }
        _ => {
            panic!("error. response should have been from the app connect")
        }
    }

    let font = load_font_from_json("../../resources/default-font.json").unwrap();
    let mut surf:DrawingSurfaceImpl = DrawingSurfaceImpl {
        appid,
        winid,
        client,
        font,
    };

    let mut hbox:HBox = HBox::make();
    let mut button1: ActionButton = ActionButton::make();
    button1._caption = "Prev".to_string();
    hbox.add(button1);
    let mut button2: ActionButton = ActionButton::make();
    button2._caption = "Play".to_string();
    hbox.add(button2);
    let mut button3: ActionButton = ActionButton::make();
    button3._caption = "Next >".to_string();
    hbox.add(button3);

    hbox.layout(&mut surf, &bounds.size());
    hbox.draw(&mut surf);

    for cmd in &surf.client.rx {
        info!("got an event {:?}",cmd);
    }
    println!("CLIENT APP ending");

// hbox
    // action button 1: Prev
    // action button 2: Play
    // action button 3: Next
    // font icon
    // hspacer
    // dropdown button

    /*

    // really fill text inside the drawing surface using a JSON font.
    - use 8x8 squares as backup
    - really do metrics
    - move UIView and DrawingSurface to core.rs
    - move ActionButton and HBox to components.rs
    - listen for mouse events, make buttons clickable using active color
    - add action events from the mouse events to app code
    - create BaseUIView and BaseParentUIView in components.rs
    - implement the hbox algorithm using spacers, hflex, vflex, and valign
     */
}
