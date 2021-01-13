use synth_generator::{prelude::*, GeneratorState};

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use std::rc::Rc;

use super::compiler::Scope;

pub type GeneratorOutput<G: Generator> = GeneratorState<G::Yield, G::Return>;

pub type UniqueId = u64;

struct Marker {
    left: usize,
    right: usize,
    ranges: BTreeMap<UniqueId, Range<usize>>,
}

impl From<Range<usize>> for Marker {
    fn from(r: Range<usize>) -> Self {
        Self {
            left: r.start,
            right: r.end,
            ranges: BTreeMap::new(),
        }
    }
}

impl Marker {
    fn next_for(&mut self, range_id: UniqueId) -> Option<Option<usize>> {
        let range = self.range();
        self.ranges.get_mut(&range_id).map(|curr| {
            if curr.start == curr.end {
                *curr = range;
            }
            curr.next()
        })
    }

    fn range(&self) -> Range<usize> {
        self.left..self.right
    }

    fn new_range(&mut self) -> UniqueId {
        let next_id = self.ranges.last_key_value().map(|(k, _)| *k).unwrap_or(0) + 1;
        self.ranges.insert(next_id, self.range());
        next_id
    }

    fn flush(&mut self) {
        self.left = self.right;
    }

    fn flush_to(&mut self, right: usize) {
        self.right = right;
        self.flush();
    }

    fn mark(&mut self, curr: usize) {
        self.right = curr;
        let range = self.range();
        self.ranges.values_mut().for_each(|value| {
            *value = range.clone();
        });
    }
}

/// TODO: Garbage collection; bounding range on Markers;
pub struct Buffered<G: Generator> {
    inner: G,
    buf: Vec<GeneratorOutput<G>>,
    markers: BTreeMap<UniqueId, Marker>,
}

impl<G> Buffered<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    fn new(inner: G) -> Self {
        Self {
            inner,
            buf: Vec::new(),
            markers: BTreeMap::new(),
        }
    }

    fn next_for(
        &mut self,
        marker_id: UniqueId,
        range_id: UniqueId,
    ) -> Option<Option<GeneratorOutput<G>>> {
        let next_idx = self
            .markers
            .get_mut(&marker_id)
            .and_then(|task| task.next_for(range_id))?;
        debug!(
            target: "compiler",
            "view next for marker={} range={} at pos={}",
            marker_id,
            range_id,
            next_idx.map(|idx| idx.to_string()).unwrap_or("unknown".to_string())
        );
        Some(next_idx.map(|idx| self.buf.get(idx).unwrap().clone()))
    }

    fn new_range(&mut self, marker_id: UniqueId) -> Option<UniqueId> {
        self.markers
            .get_mut(&marker_id)
            .map(|task| task.new_range())
    }

    fn new_marker(&mut self) -> UniqueId {
        let next_id = self.markers.last_key_value().map(|(k, _)| *k).unwrap_or(0) + 1;
        let idx = self.buf.len();
        self.markers.insert(next_id, Marker::from(idx..idx));
        next_id
    }

    fn flush(&mut self, marker_id: UniqueId) -> Option<()> {
        let curr = self.buf.len();
        self.markers
            .get_mut(&marker_id)
            .map(|marker| marker.flush_to(curr))
    }

    fn mark(&mut self, marker_id: UniqueId) -> Option<()> {
        let curr = self.buf.len();
        self.markers
            .get_mut(&marker_id)
            .map(|marker| marker.mark(curr))
    }
}

impl<G> Generator for Buffered<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        let next = self.inner.next(rng);
        self.buf.push(next.clone());
        next
    }
}

pub struct Driver<G: Generator> {
    src: Rc<RefCell<Buffered<G>>>,
}

impl<G> Generator for Driver<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        self.src.borrow_mut().next(rng)
    }
}

pub struct Cursored<G: Generator> {
    src: Rc<RefCell<Buffered<G>>>,
    marker_id: UniqueId,
}

impl<G> Cursored<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    pub fn view(&self) -> View<G> {
        let marker_id = self.marker_id;
        let range_id = self.src.borrow_mut().new_range(marker_id).unwrap();
        View {
            src: self.src.clone(),
            marker_id,
            range_id,
        }
    }

    pub fn subset(&self) -> Self {
        let marker_id = self.src.borrow_mut().new_marker();
        Self {
            src: self.src.clone(),
            marker_id,
        }
    }

    pub fn flush(&mut self) {
        self.src.borrow_mut().flush(self.marker_id).unwrap();
    }

    pub fn mark(&mut self) {
        self.src.borrow_mut().mark(self.marker_id).unwrap();
    }
}

impl<G> Clone for Cursored<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    fn clone(&self) -> Self {
        Cursored {
            src: self.src.clone(),
            marker_id: self.marker_id,
        }
    }
}

pub struct View<G: Generator> {
    src: Rc<RefCell<Buffered<G>>>,
    marker_id: UniqueId,
    range_id: UniqueId,
}

impl<G> Generator for View<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    type Yield = G::Yield;

    type Return = Option<G::Return>;

    fn next(&mut self, _rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        let output = self
            .src
            .borrow_mut()
            .next_for(self.marker_id, self.range_id)
            .unwrap();
        match output {
            Some(GeneratorState::Yielded(yielded)) => GeneratorState::Yielded(yielded),
            Some(GeneratorState::Complete(complete)) => GeneratorState::Complete(Some(complete)),
            None => GeneratorState::Complete(None),
        }
    }
}

pub fn channel<G>(generator: G) -> (Driver<G>, Cursored<G>)
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    let mut buffered = Buffered::new(generator);
    let marker_id = buffered.new_marker();
    let src = Rc::new(RefCell::new(buffered));
    let handler = Cursored {
        src: src.clone(),
        marker_id,
    };
    let driver = Driver { src };
    (driver, handler)
}

pub struct Scoped<G: Generator> {
    pub(super) cursors: HashMap<Scope, Cursored<G>>,
    pub(super) drivers: HashMap<Scope, Driver<G>>,
    pub(super) order: Vec<Scope>,
    pub(super) src: Box<G>,
    pub(super) is_complete: bool,
}

impl<G> Generator for Scoped<G>
where
    G: Generator,
    GeneratorOutput<G>: Clone,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        if self.is_complete {
            debug!("> scoped generator cycle BEGIN");
            self.is_complete = false;
            for scope in self.order.iter() {
                let cursors = self
                    .cursors
                    .iter_mut()
                    .filter(|(target, _)| target.as_in(scope).is_some())
                    .map(|(scope, cursor)| {
                        debug!(">> flush {}", scope);
                        cursor.flush();
                        (scope, cursor)
                    })
                    .collect::<Vec<_>>();
                debug!(">>> drive {}", scope);
                self.drivers.get_mut(scope).unwrap().complete(rng);
                cursors.into_iter().for_each(|(scope, cursor)| {
                    debug!(">> mark {}", scope);
                    cursor.mark()
                });
            }
        }
        let next = self.src.next(rng);
        if next.is_complete() {
            self.is_complete = true;
            debug!("< scoped generator cycle END");
        }
        next
    }
}
