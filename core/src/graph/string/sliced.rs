use crate::graph::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

pub fn slice(what: String, len: String) -> String {
    // what.truncate(len as usize);
    if let Some((l, r)) = len.split_once(':') {
        let li = l.parse::<usize>().unwrap();
        let ri = if r.is_empty() {
            what.len()
        } else {
            r.parse::<usize>().unwrap()
        };

        what[li..ri].to_string()
    } else {
        what
    }
}

type Slicer = MapOk<Rc<RefCell<StringGenerator>>, Box<dyn Fn(String) -> String>, String>;

type SlicedInner = TryYield<AndThenTry<StringGenerator, Box<dyn Fn(String) -> Slicer>, Slicer>>;

derive_generator! {
    yield String,
    return Result<String, Error>,
    pub struct Sliced(SlicedInner);
}

impl Sliced {
    pub(crate) fn new(content: StringGenerator, slices: StringGenerator) -> Self {
        let length = Rc::new(RefCell::new(slices));
        let slicer = Box::new(move |s: String| {
            let do_slice =
                Box::new(move |len| slice(s.clone(), len)) as Box<dyn Fn(String) -> String>;
            length.clone().map_ok(do_slice)
        }) as Box<dyn Fn(String) -> Slicer>;
        let out = content.and_then_try(slicer).try_yield();
        Self(out)
    }
}
