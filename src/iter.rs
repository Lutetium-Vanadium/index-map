use super::{IndexMap, OptionIndex};
use core::fmt;
use core::iter::{Enumerate, IntoIterator, Iterator};
use core::slice;

pub struct Iter<'a, T> {
    inner: Enumerate<slice::Iter<'a, OptionIndex<T>>>,
}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, item)) = self.inner.next() {
            if let OptionIndex::Some(val) = item {
                return Some((i, val));
            }
        }
        None
    }
}

impl<'a, T> IntoIterator for &'a IndexMap<T> {
    type Item = (usize, &'a T);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            inner: self.data.iter().enumerate(),
        }
    }
}

#[derive(Debug)]
pub struct IterMut<'a, T> {
    inner: Enumerate<slice::IterMut<'a, OptionIndex<T>>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, item)) = self.inner.next() {
            if let OptionIndex::Some(val) = item {
                return Some((i, val));
            }
        }
        None
    }
}

impl<'a, T> IntoIterator for &'a mut IndexMap<T> {
    type Item = (usize, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            inner: self.data.iter_mut().enumerate(),
        }
    }
}

#[derive(Clone)]
pub struct IntoIter<T> {
    inner: Enumerate<alloc::vec::IntoIter<OptionIndex<T>>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = (usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, item)) = self.inner.next() {
            if let OptionIndex::Some(item) = item {
                return Some((i, item));
            }
        }
        None
    }
}

impl<T> IntoIterator for IndexMap<T> {
    type Item = (usize, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.data.into_iter().enumerate(),
        }
    }
}

pub struct Keys<'a, T> {
    inner: Iter<'a, T>,
}

impl<T> Clone for Keys<'_, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, T> fmt::Debug for Keys<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T> Iterator for Keys<'a, T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.0)
    }
}

pub struct Values<'a, T> {
    inner: Iter<'a, T>,
}

impl<T> Clone for Values<'_, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for Values<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.1)
    }
}

#[derive(Debug)]
pub struct ValuesMut<'a, T> {
    inner: IterMut<'a, T>,
}

impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.1)
    }
}

impl<T> IndexMap<T> {
    pub fn keys(&self) -> Keys<'_, T> {
        Keys { inner: self.iter() }
    }

    pub fn values(&self) -> Values<'_, T> {
        Values { inner: self.iter() }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, T> {
        ValuesMut {
            inner: self.iter_mut(),
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        <&IndexMap<T>>::into_iter(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        <&mut IndexMap<T>>::into_iter(self)
    }
}
