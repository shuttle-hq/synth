use synth_gen::{prelude::*, GeneratorState};

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use bimap::BiBTreeMap;

use super::compiler::Scope;

pub type GeneratorOutput<G: Generator> = GeneratorState<G::Yield, G::Return>;

pub type UniqueId = u64;
#[derive(Debug, Clone)]
struct UniqueRange<Idx> {
    id: UniqueId,
    range: Range<Idx>,
}

impl Eq for UniqueRange<usize> { }

impl PartialEq for UniqueRange<usize> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for UniqueRange<usize> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UniqueRange<usize> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.range.start, self.id).cmp(&(other.range.start, other.id))
    }
}

#[derive(Clone, Debug)]
struct UniqueMarker {
    id: UniqueId,
    marker: Marker,
}

impl Eq for UniqueMarker { }

impl PartialEq for UniqueMarker {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for UniqueMarker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UniqueMarker {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_left = self
            .marker
            .ranges
            .right_values()
            .next()
            .map(|value| value.range.start);
        let other_left = other
            .marker
            .ranges
            .right_values()
            .next()
            .map(|value| value.range.start);
        (self_left, self.id).cmp(&(other_left, other.id))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
struct Marker {
    left: usize,
    right: usize,
    ranges: BiBTreeMap<UniqueId, UniqueRange<usize>>,
}

impl From<Range<usize>> for Marker {
    fn from(r: Range<usize>) -> Self {
        Self {
            left: r.start,
            right: r.end,
            ranges: BiBTreeMap::new(),
        }
    }
}

impl Marker {
    fn next_for(&mut self, range_id: UniqueId) -> Option<Option<usize>> {
        match self.ranges.get_by_left(&range_id).cloned() {
            Some(mut curr) => {
                if curr.range.start == curr.range.end {
                    curr.range = self.range();
                }
                let next = curr.range.next();
                self.ranges.insert(range_id, curr);
                Some(next)
            }
            None => None,
        }
    }

    fn range(&self) -> Range<usize> {
        self.left..self.right
    }

    fn new_range(&mut self) -> UniqueId {
        let next_id = self.ranges.iter().last().map(|(k, _)| *k).unwrap_or(0) + 1;
        self.ranges.insert(
            next_id,
            UniqueRange {
                id: next_id,
                range: self.range(),
            },
        );
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
        let mut updated_ranges = BiBTreeMap::new();
        self.ranges.right_values().for_each(|value| {
            updated_ranges.insert(
                value.id,
                UniqueRange {
                    id: value.id,
                    range: self.range(),
                },
            );
        });
        self.ranges = updated_ranges;
    }
}

pub struct Buffered<G: Generator> {
    inner: G,
    buf: Vec<GeneratorOutput<G>>,
    markers: BiBTreeMap<UniqueId, UniqueMarker>,
    gc_threshold: usize,
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
            markers: BiBTreeMap::new(),
            gc_threshold: 1000000,
        }
    }

    fn next_for(
        &mut self,
        marker_id: UniqueId,
        range_id: UniqueId,
    ) -> Option<Option<GeneratorOutput<G>>> {
        self.garbage_collect();
        let mut unique_marker = self.markers.get_by_left(&marker_id).cloned()?;
        let next_idx = unique_marker.marker.next_for(range_id);
        self.markers.insert(marker_id, unique_marker);
        let next_idx = next_idx?;
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
        let mut unique_marker = self.markers.get_by_left(&marker_id).cloned()?;
        let result = unique_marker.marker.new_range();
        self.markers.insert(marker_id, unique_marker);
        Some(result)
    }

    fn new_marker(&mut self) -> UniqueId {
        let next_id = self.markers.iter().last().map(|(k, _)| *k).unwrap_or(0) + 1;
        let idx = self.buf.len();
        self.markers.insert(
            next_id,
            UniqueMarker {
                id: next_id,
                marker: Marker::from(idx..idx),
            },
        );
        next_id
    }

    fn flush(&mut self, marker_id: UniqueId) -> Option<()> {
        let curr = self.buf.len();
        let mut unique_marker = self.markers.get_by_left(&marker_id).cloned()?;
        unique_marker.marker.flush_to(curr);
        self.markers.insert(marker_id, unique_marker);
        Some(())
    }

    fn mark(&mut self, marker_id: UniqueId) -> Option<()> {
        let curr = self.buf.len();
        let mut unique_marker = self.markers.get_by_left(&marker_id).cloned()?;
        unique_marker.marker.mark(curr);
        self.markers.insert(marker_id, unique_marker);
        Some(())
    }

    fn garbage_collect(&mut self) {
        let threshold = self.gc_threshold;
        let mut triggered = false;
        self.markers.right_values().next().map(|unique_marker| {
            unique_marker
                .marker
                .ranges
                .right_values()
                .next()
                .map(|unique_range| {
                    triggered = unique_range.range.start > threshold;
                });
        });
        if triggered {
            let mut updated_markers = BiBTreeMap::new();
            self.markers.right_values().for_each(|unique_marker| {
                let mut updated_marker = unique_marker.clone();
                let mut updated_ranges = BiBTreeMap::new();
                unique_marker
                    .marker
                    .ranges
                    .right_values()
                    .for_each(|value| {
                        updated_ranges.insert(
                            value.id,
                            UniqueRange {
                                id: value.id,
                                range: value.range.start - threshold..value.range.end - threshold,
                            },
                        );
                    });
                let marker = &mut updated_marker.marker;
                if marker.right < threshold {
                    marker.flush_to(0);
                } else {
                    marker.right = marker.right - threshold;
                    assert!(marker.left <= marker.right);
                }
                marker.ranges = updated_ranges;
                updated_markers.insert(updated_marker.id, updated_marker);
            });
            self.markers = updated_markers;
            assert!(self.buf.len() >= threshold);
            self.buf.drain(0..threshold);
        }
    }

    #[allow(dead_code)]
    fn set_gc_threshold(&mut self, new_threshold: usize) {
        self.gc_threshold = new_threshold;
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

#[cfg(test)]
pub mod tests {
    use rand::RngCore;
    use std::borrow::{Borrow, BorrowMut};
    use std::ops::Deref;

    use super::*;

    #[test]
    fn buffered_gc() {
        let mut rng = rand::thread_rng();
        let initial_buf_len = 100;
        // Single view
        for threshold in vec![0, 1, 10] {
            let seed = 1.yield_token();
            let (mut driver, mut cursor) = channel(seed);
            let mut view = cursor.view();
            driver.src.deref().borrow_mut().set_gc_threshold(threshold);
            // This fills up the buffer
            cursor.flush();
            for _ in 0..initial_buf_len {
                driver.next(&mut rng);
            }
            assert_eq!(driver.src.deref().borrow().buf.len(), initial_buf_len);
            cursor.mark();
            // This advances `view`'s cursor in buffer, consuming its values
            for _ in 0..initial_buf_len {
                view.next(&mut rng);
            }
            assert!(
                (threshold > 0 && driver.src.deref().borrow().buf.len() < initial_buf_len) ||
                (threshold == 0 && driver.src.deref().borrow().buf.len() == initial_buf_len)
            );
        }
        // Multiple views
        for threshold in vec![0, 1, 10] {
            struct TestView<G> where
                G: Generator,
            {
                view: Rc<RefCell<View<G>>>,
                pos: usize,
            }
            let seed = 1.yield_token();
            let (mut driver, mut cursor) = channel(seed);
            let mut views = vec![];
            let mut view_pos = vec![];
            for _ in 0..10 {
                let view = Rc::new(RefCell::new(cursor.view()));
                view_pos.push(TestView {
                    view: view.clone(),
                    pos: 0,
                });
                views.push(view);
            }
            driver.src.deref().borrow_mut().set_gc_threshold(threshold);
            // This fills up the buffer
            cursor.flush();
            for _ in 0..initial_buf_len {
                driver.next(&mut rng);
            }
            cursor.mark();
            while view_pos.len() > 0 {
                let i: usize = (rng.next_u32() as usize) % view_pos.len();
                view_pos[i].pos += 1;
                view_pos[i].borrow_mut().view.deref().borrow_mut().next(&mut rng);
                if view_pos[i].borrow().pos >= driver.src.deref().borrow().buf.len() - threshold {
                    view_pos.remove(i);
                }
            }
            assert!((threshold > 0 && driver.src.deref().borrow().buf.len() < initial_buf_len) ||
                    (threshold == 0 && driver.src.deref().borrow().buf.len() == initial_buf_len));
        }
    }
}
