use log::{debug, error, info, LevelFilter, Metadata, Record, set_logger};

static LOGGER:SimpleLogger = SimpleLogger;
fn main() {
    set_logger(&LOGGER).map(|()|log::set_max_level(LevelFilter::Info));
    println!("Hello, world!");

    info!("this is an info");
    error!("this is an error");
    debug!("this is a debug");

    println!("and now we are done");
}


struct SimpleLogger;
impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        println!("logger getting metadata {:?}",metadata);
        return true;
    }

    fn log(&self, record: &Record) {
        // println!("Logging {:?}",record);
        let prefix = if let Some(modname) = record.module_path() {
            modname
        } else {
            "unknown"
        };
        let message = record.args().as_str().unwrap();
        println!("{}: {}", prefix, message);
    }

    fn flush(&self) {
        println!("flushing")
    }
}
