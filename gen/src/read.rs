#![allow(dead_code, missing_docs)]
//! **TODO** This module is unfinished WIP. It is required for
//! deserializing a generator into a Rust data object instance.
use crate::{Generator, GeneratorState, Rng};

use std::collections::VecDeque;

pub struct Readable<'r, G> {
    buffers: VecDeque<VecDeque<u8>>,
    inner: G,
    rng: &'r mut Rng,
}

impl<'r, G> Readable<'r, G> {
    pub fn new(generator: G, rng: &'r mut Rng) -> Self {
        Self {
            buffers: VecDeque::new(),
            inner: generator,
            rng,
        }
    }
}

impl<'r, G> Readable<'r, G>
where
    G: Generator,
    G::Yield: Into<Vec<u8>>,
{
    fn fill(&mut self, hint: usize) -> usize {
        let back = if let Some(back) = self.buffers.back_mut() {
            back
        } else {
            self.buffers.push_back(VecDeque::new());
            self.buffers.back_mut().unwrap()
        };

        while hint > back.len() {
            match self.inner.next(self.rng) {
                GeneratorState::Complete(_) => break,
                GeneratorState::Yielded(y) => back.extend(y.into()),
            }
        }

        let len = back.len();

        if back.is_empty() {
            self.buffers.pop_back().unwrap();
        }

        len
    }

    fn draw(&mut self, hint: usize) -> Vec<u8> {
        let front = if let Some(front) = self.buffers.front_mut() {
            front
        } else {
            self.buffers.push_front(VecDeque::new());
            self.buffers.front_mut().unwrap()
        };

        let mut buf = Vec::new();
        let hint = std::cmp::min(hint, front.len());
        buf.extend(front.drain(..hint));

        if front.is_empty() {
            self.buffers.pop_front().unwrap();
        }

        buf
    }
}

impl<'r, G> std::io::Read for Readable<'r, G>
where
    G: Generator,
    G::Yield: Into<Vec<u8>>,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let hint = buf.len();

        if self.buffers.len() < 2 && self.buffers.pop_front().map(|b| b.len()).unwrap_or(0) < hint {
            self.fill(hint);
        }

        let draw = self.draw(hint);

        let len_draw = draw.len();
        buf[..len_draw].clone_from_slice(draw.as_slice());
        Ok(len_draw)
    }
}

// TODO: tests
#[cfg(test)]
pub mod tests {
    use super::*;

    use std::io::Read;

    use crate::value::{Primitive, Token};

    fn readable() {
        let rng = &mut rand::thread_rng();

        let token = Token::Primitive(Primitive::String("hey ho".to_string()));
        let readable = Readable::new(token.just(), rng);

        let mut out = String::new();
        //readable.read_to_string(&mut out).unwrap();

        println!("{}", out);

        assert_eq!(out, "hey ho".to_string())
    }
}
