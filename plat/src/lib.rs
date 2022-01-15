use common::Rect;
#[cfg(target_os="macos")]
pub use native_macos::Plat as Plat;
#[cfg(target_os="linux")]
pub use native_linux::Plat as Plat;




