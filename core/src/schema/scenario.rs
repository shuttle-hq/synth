use std::path::PathBuf;

use crate::Namespace;

pub struct Scenario {
    namespace: Namespace,
    path: PathBuf,
}

impl Scenario {
    pub fn new(namespace: Namespace, path: PathBuf) -> Self {
        Self { namespace, path }
    }

    pub fn build(self) -> Namespace {
        self.namespace
    }
}
