use log::{LevelFilter, Metadata, Record};

pub struct CompositeLogger {
    loggers: Vec<Box<dyn log::Log>>,
}

impl CompositeLogger {
    pub fn init(loggers: Vec<Box<dyn log::Log>>) {
        let cl = Self { loggers };
        log::set_boxed_logger(Box::new(cl)).expect("Could not set Composite logger");
        log::set_max_level(LevelFilter::Trace)
    }
}

impl log::Log for CompositeLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        self.loggers.iter().for_each(|logger| logger.log(record))
    }

    fn flush(&self) {
        self.loggers.iter().for_each(|logger| logger.flush())
    }
}
