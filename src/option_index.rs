#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Copy)]
pub(crate) enum OptionIndex<T> {
    Some(T),
    Index(usize),
    NoIndex,
}

use OptionIndex::*;

impl<T: Clone> Clone for OptionIndex<T> {
    fn clone(&self) -> Self {
        match self {
            Some(t) => Some(t.clone()),
            Index(i) => Index(*i),
            NoIndex => NoIndex,
        }
    }
}

impl<T> OptionIndex<T> {
    pub(crate) fn is_inner(&self) -> bool {
        matches!(self, Some(_))
    }

    pub(crate) fn as_ref(&self) -> OptionIndex<&T> {
        match self {
            Some(ref t) => Some(t),
            Index(i) => Index(*i),
            NoIndex => NoIndex,
        }
    }

    pub(crate) fn as_mut(&mut self) -> OptionIndex<&mut T> {
        match self {
            Some(ref mut t) => Some(t),
            Index(i) => Index(*i),
            NoIndex => NoIndex,
        }
    }

    pub(crate) fn take(&mut self) -> OptionIndex<T> {
        let mut val = NoIndex;
        core::mem::swap(self, &mut val);
        val
    }

    pub(crate) fn into_inner(self) -> Option<T> {
        match self {
            Some(t) => Option::Some(t),
            Index(_) => None,
            NoIndex => None,
        }
    }

    pub(crate) fn into_index(self) -> Option<usize> {
        match self {
            Index(i) => Option::Some(i),
            Some(_) => None,
            NoIndex => None,
        }
    }
}
