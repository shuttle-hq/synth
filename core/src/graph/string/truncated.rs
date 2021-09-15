use crate::graph::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

pub fn truncate(mut what: String, len: u64) -> String {
    what.truncate(len as usize);
    what
}

type Truncator = MapOk<Rc<RefCell<SizeGenerator>>, Box<dyn Fn(u64) -> String>, String>;

type TruncatedInner =
    TryYield<AndThenTry<StringGenerator, Box<dyn Fn(String) -> Truncator>, Truncator>>;

derive_generator! {
    yield String,
    return Result<String, Error>,
    pub struct Truncated(TruncatedInner);
}

impl Truncated {
    pub(crate) fn new(content: StringGenerator, length: SizeGenerator) -> Self {
        let length = Rc::new(RefCell::new(length));
        let truncator = Box::new(move |s: String| {
            let do_truncate =
                Box::new(move |len| truncate(s.clone(), len)) as Box<dyn Fn(u64) -> String>;
            length.clone().map_ok(do_truncate)
        }) as Box<dyn Fn(String) -> Truncator>;
        let out = content.and_then_try(truncator).try_yield();
        Self(out)
    }
}
