use audio::AudioService;
use db::JDB;
use crate::{App, Debugger, WM};

pub struct CentralState {
    pub(crate) wms:Vec<WM>,
    pub(crate) apps:Vec<App>,
    pub(crate) debuggers:Vec<Debugger>,
    pub(crate) db:JDB,
    pub(crate) audio_service:AudioService,
}
