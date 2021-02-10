//! A Map with automatically generated `usize`s as keys.
//!
//! It supports _most_ of the helper functions present on [`HashMap`](std::collections::HashMap),
//! but fundamentally stores all data in a [`Vec`]

#![no_std]
// #![warn(missing_docs)]
#![warn(rust_2018_idioms)]

extern crate alloc;

use alloc::vec::Vec;

mod iter;
mod option_index;
use option_index::OptionIndex;

/// A map of `usize` to value, which allows efficient O(1) indexing, and O(1) popping.
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct IndexMap<T> {
    data: Vec<OptionIndex<T>>,
    head: Option<usize>,
}

impl<T> IndexMap<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            head: None,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            head: None,
        }
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn len(&self) -> usize {
        if self.head.is_none() {
            return self.data.len();
        }

        self.data.iter().fold(0, |acc, option_index| {
            acc + (option_index.is_inner() as usize)
        })
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.data.clear()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit()
    }

    pub fn contains_key(&self, index: usize) -> bool {
        if index >= self.data.len() {
            return false;
        }

        self.data[index].is_inner()
    }

    pub fn insert(&mut self, value: T) -> usize {
        if let Some(head) = self.head {
            self.head = self.data[head].take().into_index();
            self.data[head] = OptionIndex::Some(value);
            head
        } else {
            self.data.push(OptionIndex::Some(value));
            self.data.len()
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if !self.data.get_mut(index)?.is_inner() {
            return None;
        }

        let val = self.data.get_mut(index)?.take().into_inner()?;

        if let Some(head) = self.head {
            self.data[index] = OptionIndex::Index(head);
        } else {
            self.data[index] = OptionIndex::NoIndex;
        }

        self.head = Some(index);

        Some(val)
    }

    pub fn remove_entry(&mut self, index: usize) -> Option<(usize, T)> {
        Some((index, self.remove(index)?))
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)?.as_ref().into_inner()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)?.as_mut().into_inner()
    }

    pub fn retain<P>(&mut self, mut predicate: P)
    where
        P: FnMut(usize, &mut T) -> bool,
    {
        for (i, v) in self.data.iter_mut().enumerate() {
            if let OptionIndex::Some(val) = v {
                if !predicate(i, val) {
                    *v = if let Some(head) = self.head {
                        OptionIndex::Index(head)
                    } else {
                        OptionIndex::NoIndex
                    };

                    self.head = Some(i)
                }
            }
        }
    }
}

impl<T: Clone> Clone for IndexMap<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            head: self.head,
        }
    }
}

impl<T> Default for IndexMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexMap, OptionIndex as OI};

    fn assert_state<T: Eq + core::fmt::Debug>(
        map: &IndexMap<T>,
        data: &[OI<T>],
        head: Option<usize>,
    ) {
        assert_eq!(map.data[..], data[..]);
        assert_eq!(map.head, head);
    }

    #[test]
    fn test_map() {
        let mut map = IndexMap::new();

        let _ = map.insert('a');
        let b = map.insert('b');
        let c = map.insert('c');
        let _ = map.insert('d');
        let e = map.insert('e');

        assert_state(
            &map,
            &[
                OI::Some('a'),
                OI::Some('b'),
                OI::Some('c'),
                OI::Some('d'),
                OI::Some('e'),
            ],
            None,
        );

        assert_eq!(map.remove(b), Some('b'));
        assert_state(
            &map,
            &[
                OI::Some('a'),
                OI::NoIndex,
                OI::Some('c'),
                OI::Some('d'),
                OI::Some('e'),
            ],
            Some(1),
        );

        assert_eq!(map.remove(e), Some('e'));
        assert_state(
            &map,
            &[
                OI::Some('a'),
                OI::NoIndex,
                OI::Some('c'),
                OI::Some('d'),
                OI::Index(1),
            ],
            Some(4),
        );

        assert_eq!(map.remove(c), Some('c'));
        assert_state(
            &map,
            &[
                OI::Some('a'),
                OI::NoIndex,
                OI::Index(4),
                OI::Some('d'),
                OI::Index(1),
            ],
            Some(2),
        );

        map.shrink_to_fit();
        assert_state(
            &map,
            &[OI::Some('a'), OI::NoIndex, OI::Index(1), OI::Some('d')],
            Some(2),
        );
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut map = IndexMap::new();

        let a = map.insert('a');
        let b = map.insert('b');
        let c = map.insert('c');
        let d = map.insert('d');
        let e = map.insert('e');

        map.remove(e);
        map.remove(b);
        map.remove(d);

        assert_state(
            &map,
            &[
                OI::Some('a'),
                OI::Index(4),
                OI::Some('c'),
                OI::Index(1),
                OI::NoIndex,
            ],
            Some(3),
        );

        map.shrink_to_fit();

        assert_state(&map, &[OI::Some('a'), OI::NoIndex, OI::Some('c')], Some(1));

        map.remove(c);
        map.shrink_to_fit();

        assert_state(&map, &[OI::Some('a')], None);

        map.remove(a);
        assert_state(&map, &[OI::NoIndex], Some(0));

        map.shrink_to_fit();
        assert_state(&map, &[], None);

        let mut map = IndexMap::new();

        let _ = map.insert('a');
        let b = map.insert('b');
        let _ = map.insert('c');
        let d = map.insert('d');
        let e = map.insert('e');

        map.remove(b);
        map.remove(d);
        map.remove(e);

        assert_state(
            &map,
            &[
                OI::Some('a'),
                OI::NoIndex,
                OI::Some('c'),
                OI::Index(1),
                OI::Index(3),
            ],
            Some(4),
        );

        map.shrink_to_fit();

        assert_state(&map, &[OI::Some('a'), OI::NoIndex, OI::Some('c')], Some(1));
    }
}
