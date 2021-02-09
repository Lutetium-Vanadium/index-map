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

#[cfg(test)]
mod tests {
    use super::OptionIndex;
    use alloc::vec;

    fn make_some<T>(t: T) -> OptionIndex<T> {
        OptionIndex::Some(t)
    }
    fn make_idx(idx: usize) -> OptionIndex<usize> {
        OptionIndex::Index(idx)
    }
    fn make_noidx() -> OptionIndex<usize> {
        OptionIndex::NoIndex
    }

    #[test]
    fn test_is_inner() {
        assert!(make_some(0).is_inner());
        assert!(!make_idx(0).is_inner());
        assert!(!make_noidx().is_inner());
    }

    #[test]
    fn test_as_ref() {
        let opt = make_some(vec![0, 1]);
        assert_eq!(opt.as_ref().into_inner().unwrap()[..], [0, 1]);

        assert_eq!(make_idx(1).as_ref(), OptionIndex::Index(1));
        assert_eq!(make_noidx().as_ref(), OptionIndex::NoIndex);
    }

    #[test]
    fn test_as_mut() {
        let mut opt = make_some(0);
        *opt.as_mut().into_inner().unwrap() = 1;
        assert_eq!(opt.into_inner().unwrap(), 1);

        assert_eq!(make_idx(1).as_mut(), OptionIndex::Index(1));
        assert_eq!(make_noidx().as_mut(), OptionIndex::NoIndex)
    }

    #[test]
    fn test_take() {
        for i in vec![make_some(0), make_idx(1), make_noidx()] {
            let mut opt = i;
            assert_eq!(opt.take(), i);
            assert_eq!(opt, make_noidx());
        }
    }

    #[test]
    fn test_into_inner() {
        assert_eq!(make_some(2).into_inner(), Some(2));
        assert_eq!(make_idx(0).into_inner(), None);
        assert_eq!(make_noidx().into_inner(), None);
    }

    #[test]
    fn test_into_index() {
        assert_eq!(make_idx(2).into_index(), Some(2));
        assert_eq!(make_some(0).into_index(), None);
        assert_eq!(make_noidx().into_index(), None);
    }
}
