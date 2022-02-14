use log::{Metadata, Record};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub struct CoolLogger;
impl log::Log for CoolLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        println!("logger getting metadata {:?}",metadata);
        return true;
    }

    fn log(&self, record: &Record) {
        // println!("Logging {:?}",record);
        let prefix = if let Some(modname) = record.module_path() {
            modname.to_uppercase()
        } else {
            "unknown".to_uppercase()
        };
        println!("{}: {}", prefix, record.args());
    }

    fn flush(&self) {
        println!("flushing")
    }
}

