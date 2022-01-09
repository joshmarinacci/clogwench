/*
- make screen and graphics abstraction with separate testing and linux impulse
	- Build the rect abstraction l. Use it for drawing rects
	    and window bounds and deciding which
	    window gets the mouse events.
	- create a virtual surface used for testing. then the
	    linux-wm can have multiple gfx implementations.
	    common screen and surface impls.
 */

use crate::{ARGBColor, Rect};
/*
pub trait Screen {
    fn get_bounds(&self) -> &Rect;
}
pub trait Surface {
    fn get_bounds(&self) -> &Rect;
    fn fill_rect(&self, rect:&Rect, color:&ARGBColor);
}
*/
