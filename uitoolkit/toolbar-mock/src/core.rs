use log::info;
use common::{APICommand, ARGBColor, BLACK, DrawImageCommand, DrawRectCommand, Point, Rect, Size};
use common::graphics::{GFXBuffer, PixelLayout};
use uuid::Uuid;
use common::client::ClientConnection;
use common::events::MouseDownEvent;
use common::font::FontInfo2;
use crate::components::ActionButton;

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
    down:bool,
    root:dyn UIView,
}

pub struct PointerEvent {

}

impl DrawingSurfaceImpl {
    pub(crate) fn poll_input(&mut self) {
        for cmd in &self.client.rx {
            info!("got an event {:?}",cmd);
            match cmd {
                // APICommand::KeyDown(kd) => {
                //     info!("got a keydown event");
                //     match kd.key {
                //         KeyCode::ARROW_RIGHT => px += 1,
                //         KeyCode::ARROW_LEFT => px -= 1,
                //         KeyCode::ARROW_UP => py -= 1,
                //         KeyCode::ARROW_DOWN => py += 1,
                //         _ => {}
                //     }
                //     redraw(&client, appid, winid, bounds, px, py, &pattern_buffer, &text_buffer);
                //     client.send(APICommand::Debug(DebugMessage::AppLog(String::from("got-keyboard-event"))));
                // }
                APICommand::MouseDown(md) => {
                    info!("got a mouse down event {},{}",md.x,md.y);
                    self.handle_mouse_down(md);
                    // client.send(APICommand::Debug(DebugMessage::AppLog(String::from("got-mouse-event"))));
                }
                APICommand::SystemShutdown => {
                    info!("CLIENT app:  system is shutting down. bye!");
                    // client.send(APICommand::Debug(DebugMessage::AppLog(String::from("got-shutdown"))));
                    break;
                }
                _ => {}
            }
        }

    }
    fn handle_mouse_down(&mut self, e: MouseDownEvent) {
        info!("handling mouse down {:?}",e);
        //            this.down = true;
        self.down = true;
        //             let position = this.surface.screen_to_local(domEvent)
        let position = Point::init(e.x,e.y);
        info!("mouse position {}",position);
        //             this.last_point = position
        //             this.path = this.scan_path(position)
        let path = self.scan_path(position);
        //             this.target = this.path[this.path.length-1] // last
        //             let evt = new PointerEvent()
        let evt = PointerEvent {

        };
        //             evt.button = domEvent.button
        evt.button = e.button;
        //             evt.type = POINTER_DOWN
        evt.type = POINTER_DOWN;
        //             evt.category = POINTER_CATEGORY
        evt.category = POINTER_CATEGORY;
        //             evt.position = position
        evt.position = position;
        //             evt.ctx = this.surface
        //             evt.direction = "down"
        evt.direction = EVENT_DIRECTION_DOWN;
        //             evt.target = this.target
        //             this.propagatePointerEvent(evt,this.path)
        self.propagate_pointer_event(evt,path);
        //             this.surface.repaint()
        //             domEvent.preventDefault()
    }
    fn scan_path(&self, point: Point) -> _ {
        return vec![&self.root]
    }
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
    fn name(&self) -> &str;
    fn size(&self) -> Size;
    fn position(&self) -> Point;
    fn layout(&mut self, g:&impl DrawingSurface, available:&Size) -> Size;
    fn draw(&self, g:&impl DrawingSurface);
}

pub fn rect_from_view(view: &&ActionButton) -> Rect {
    Rect::from_ints(view.position().x,view.position().y,view.size().w,view.size().h)
}
