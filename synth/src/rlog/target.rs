use log::{Metadata, Record};

pub struct TargetLogger<L> {
    target: String,
    inner: L,
}

impl<L: log::Log> TargetLogger<L> {
    pub(crate) fn new(target: String, inner: L) -> Self {
        Self { target, inner }
    }
}

impl<L: log::Log> log::Log for TargetLogger<L> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        return metadata.target().to_lowercase() == self.target;
    }

    fn log(&self, record: &Record) {
        if !(self.enabled(record.metadata())) {
            return;
        }
        self.inner.log(record)
    }

    fn flush(&self) {
        self.inner.flush()
    }
}
