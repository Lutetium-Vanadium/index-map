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
