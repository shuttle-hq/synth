use crate::{Generator, GeneratorState, Rng};

use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};

impl<G> Generator for Rc<RefCell<G>>
where
    G: Generator,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        self.borrow_mut().next(rng)
    }
}

pub type DependentId = u16;

pub type GeneratedQueue<G: Generator> = VecDeque<GeneratorState<G::Yield, G::Return>>;

pub struct InnerStar<G: Generator> {
    inner: G,
    routes: BTreeMap<DependentId, GeneratedQueue<G>>,
}

impl<G> InnerStar<G>
where
    G: Generator,
{
    fn new(generator: G) -> Self {
        Self {
            inner: generator,
            routes: BTreeMap::new(),
        }
    }
}

impl<G> InnerStar<G>
where
    G: Generator,
{
    fn register_new(&mut self, with: GeneratedQueue<G>) -> DependentId {
        let next = self.routes.keys().last().map(|last| last + 1).unwrap_or(0);
        self.routes.insert(next, with);
        next
    }

    #[inline]
    fn register_default(&mut self) -> DependentId {
        self.register_new(GeneratedQueue::<G>::default())
    }

    fn unregister(
        &mut self,
        id: DependentId,
    ) -> Option<VecDeque<GeneratorState<G::Yield, G::Return>>> {
        self.routes.remove(&id)
    }
}

impl<G> InnerStar<G>
where
    G: Generator,
    G::Yield: Clone,
    G::Return: Clone,
{
    fn register_from(&mut self, id: DependentId) -> DependentId {
        let new_queue = self.routes.get(&id).unwrap().clone();
        let new_id = self.register_new(new_queue);
        new_id
    }

    fn next_for(&mut self, id: DependentId, rng: &mut Rng) -> GeneratorState<G::Yield, G::Return> {
        self.routes
            .get_mut(&id)
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| {
                let next = self.inner.next(rng);
                for route in self.routes.values_mut() {
                    route.push_back(next.clone());
                }
                self.next_for(id, rng)
            })
    }
}

pub struct Star<G: Generator>(RefCell<InnerStar<G>>);

impl<G> Star<G>
where
    G: Generator,
{
    fn new(generator: G) -> Self {
        Self(RefCell::new(InnerStar::new(generator)))
    }
}

impl<G> Star<G>
where
    G: Generator,
{
    #[allow(dead_code)]
    fn register_new(&self, with: GeneratedQueue<G>) -> DependentId {
        self.0.borrow_mut().register_new(with)
    }

    fn register_default(&self) -> DependentId {
        self.0.borrow_mut().register_default()
    }

    fn unregister(&self, id: DependentId) -> Option<VecDeque<GeneratorState<G::Yield, G::Return>>> {
        self.0.borrow_mut().unregister(id)
    }

    fn borrow_mut(&self) -> std::cell::RefMut<'_, G> {
        std::cell::RefMut::map(self.0.borrow_mut(), |star| &mut star.inner)
    }
}

impl<G> Star<G>
where
    G: Generator,
    G::Yield: Clone,
    G::Return: Clone,
{
    fn register_from(&self, id: DependentId) -> DependentId {
        self.0.borrow_mut().register_from(id)
    }

    fn next_for(&self, id: DependentId, rng: &mut Rng) -> GeneratorState<G::Yield, G::Return> {
        self.0.borrow_mut().next_for(id, rng)
    }
}

/// On next for this, it looks up inside its own queue in `Star<G>`.
/// `Star<G>` logic makes it so that if queue is exhausted, the
/// generator is driven further. Everytime the generator is driven
/// further, the registered queues are updated with the generated
/// item. This should fit expectation: cloning a `Shared<G>` will give
/// you a generator which starts from where that other one left off.
pub struct Shared<G: Generator> {
    inner: Rc<Star<G>>,
    id: DependentId,
}

impl<G> Shared<G>
where
    G: Generator,
{
    pub fn new(generator: G) -> Self {
        let inner = Rc::new(Star::new(generator));
        let id = inner.register_default();
        Self { inner, id }
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, G> {
        self.inner.borrow_mut()
    }
}

impl<G> Drop for Shared<G>
where
    G: Generator,
{
    fn drop(&mut self) {
        self.inner.unregister(self.id).unwrap();
    }
}

impl<G> Clone for Shared<G>
where
    G: Generator,
    G::Yield: Clone,
    G::Return: Clone,
{
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        let id = inner.register_from(self.id);
        Self { inner, id }
    }
}

impl<G> Generator for Shared<G>
where
    G: Generator,
    G::Yield: Clone,
    G::Return: Clone,
{
    type Yield = G::Yield;

    type Return = G::Return;

    fn next(&mut self, rng: &mut Rng) -> GeneratorState<Self::Yield, Self::Return> {
        self.inner.next_for(self.id, rng)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::GeneratorExt;
    use crate::Seed;

    #[test]
    fn shared() {
        let mut rng = rand::thread_rng();

        let gen = Seed::new::<u16>().once();

        let mut shared_1 = Shared::new(gen);
        let mut shared_2 = shared_1.clone();

        for _ in (0..5) {
            assert_eq!(shared_1.next(&mut rng), shared_2.next(&mut rng));
        }

        let mut left_next_5 = Vec::new();
        for _ in (0..5) {
            left_next_5.push(shared_1.next(&mut rng));
        }

        let mut right_next_5 = Vec::new();
        for _ in (0..5) {
            right_next_5.push(shared_2.next(&mut rng));
        }

        let mut shared_3 = shared_1.clone();

        let left_next_5_after = shared_3.take(5).complete(&mut rng);
        let right_next_5_after = shared_2.take(5).complete(&mut rng);

        assert_eq!(left_next_5, right_next_5);

        assert_eq!(left_next_5_after, right_next_5_after);
    }
}
