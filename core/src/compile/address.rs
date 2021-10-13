use std::collections::VecDeque;

use crate::schema::FieldRef;

/// A struct to hold the parsed components of the location of a node in a compiled tree.
///
/// An address of a node in the `Content` tree is an array of string names of child nodes, as
/// declared by the node's [`Compile`](super::Compile) implementation.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Address(VecDeque<String>);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut chunks = self.iter();
        write!(f, "{}", chunks.next().unwrap_or("{top-level}"))?;
        for chunk in chunks {
            write!(f, ".{}", chunk)?;
        }
        Ok(())
    }
}

impl Address {
    #[inline]
    pub fn new_root() -> Self {
        Self(VecDeque::new())
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    pub fn root(&self) -> Self {
        self.iter().map(|node| node.to_string()).take(1).collect()
    }

    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.0.iter().map(|value| value.as_str())
    }

    #[inline]
    pub fn deeper(&mut self) -> Option<String> {
        self.0.pop_front()
    }

    #[inline]
    pub fn shallower(&mut self) -> Option<String> {
        self.0.pop_back()
    }

    #[inline]
    pub fn into_shallower(mut self) -> Self {
        self.shallower().unwrap();
        self
    }

    #[inline]
    pub fn within(&mut self, scope: &str) -> &mut Self {
        self.0.push_front(scope.to_string());
        self
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn into_within(mut self, scope: &str) -> Self {
        self.within(scope);
        self
    }

    #[inline]
    pub fn at(&mut self, attribute: &str) -> &mut Self {
        self.0.push_back(attribute.to_string());
        self
    }

    #[inline]
    pub fn into_at(mut self, attribute: &str) -> Self {
        self.at(attribute);
        self
    }

    pub fn as_local_to(&self, root: &Self) -> Option<Self> {
        if root.0.len() > self.0.len() {
            None
        } else {
            Some(
                self.iter()
                    .zip(root.iter())
                    .skip_while(|(left, right)| left == right)
                    .map(|(left, _)| left.to_string())
                    .collect(),
            )
        }
    }

    pub fn as_in(&self, other: &Self) -> Option<Self> {
        let mut out = self.clone();
        for level in other.iter() {
            if *level != out.deeper()? {
                return None;
            }
        }
        Some(out)
    }

    pub fn concat(&self, other: &Self) -> Self {
        let mut out = self.clone();
        out.extend(other.clone().into_iter());
        out
    }

    pub fn relativize(&self, with: &Self) -> (Self, Self) {
        let root = self.common_root(with);
        let relative = self.as_in(&root).unwrap();
        (root, relative)
    }

    pub fn common_root(&self, other: &Self) -> Self {
        self.iter()
            .zip(other.iter())
            .take_while(|(left, right)| left == right)
            .map(|(left, _)| left.to_string())
            .collect()
    }
}

impl std::iter::FromIterator<String> for Address {
    #[inline]
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl std::iter::Extend<String> for Address {
    #[inline]
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl std::iter::IntoIterator for Address {
    type Item = String;

    type IntoIter = std::collections::vec_deque::IntoIter<String>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<FieldRef> for Address {
    #[inline]
    fn from(field: FieldRef) -> Self {
        field.into_iter().collect()
    }
}
