use common::Rect;
#[cfg(target_os="macos")]
pub use native_macos::TM as TM;
pub use native_macos::make_plat as make_plat;
#[cfg(target_os="linux")]
pub use native_linux::Plat as Plat;




