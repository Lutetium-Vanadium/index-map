use super::{IndexMap, OptionIndex};
use core::fmt;
use core::iter::{Enumerate, ExactSizeIterator, IntoIterator, Iterator};
use core::slice;

pub struct Iter<'a, T> {
    inner: Enumerate<slice::Iter<'a, OptionIndex<T>>>,
    len: usize,
}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            len: self.len,
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
                self.len -= 1;
                return Some((i, val));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

impl<'a, T> IntoIterator for &'a IndexMap<T> {
    type Item = (usize, &'a T);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            inner: self.data.iter().enumerate(),
            len: self.len(),
        }
    }
}

#[derive(Debug)]
pub struct IterMut<'a, T> {
    inner: Enumerate<slice::IterMut<'a, OptionIndex<T>>>,
    len: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (usize, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, item)) = self.inner.next() {
            if let OptionIndex::Some(val) = item {
                self.len -= 1;
                return Some((i, val));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}

impl<'a, T> IntoIterator for &'a mut IndexMap<T> {
    type Item = (usize, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            len: self.len(),
            inner: self.data.iter_mut().enumerate(),
        }
    }
}

#[derive(Clone)]
pub struct IntoIter<T> {
    inner: Enumerate<alloc::vec::IntoIter<OptionIndex<T>>>,
    len: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = (usize, T);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, item)) = self.inner.next() {
            if let OptionIndex::Some(item) = item {
                self.len -= 1;
                return Some((i, item));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> IntoIterator for IndexMap<T> {
    type Item = (usize, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len(),
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.len, Some(self.inner.len))
    }
}

impl<T> ExactSizeIterator for Keys<'_, T> {}

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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.len, Some(self.inner.len))
    }
}

impl<T> ExactSizeIterator for Values<'_, T> {}

#[derive(Debug)]
pub struct ValuesMut<'a, T> {
    inner: IterMut<'a, T>,
}

impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.next()?.1)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.inner.len, Some(self.inner.len))
    }
}

impl<T> ExactSizeIterator for ValuesMut<'_, T> {}

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

#[cfg(test)]
mod tests {
    use super::IndexMap;

    #[test]
    fn test_iter() {
        let mut map = IndexMap::new();
        let a = map.insert("a");
        let b = map.insert("b");
        let c = map.insert("c");
        map.remove(b);
        let mut iter = map.iter().map(|(i, v)| (i, *v));
        assert_eq!(iter.next(), Some((a, "a")));
        assert_eq!(iter.next(), Some((c, "c")));
        assert_eq!(iter.next(), None);

        assert_eq!(b, map.insert("b"));
        let mut iter = map.iter().map(|(i, v)| (i, *v));
        assert_eq!(iter.next(), Some((a, "a")));
        assert_eq!(iter.next(), Some((b, "b")));
        assert_eq!(iter.next(), Some((c, "c")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_mut() {
        let mut map = IndexMap::new();
        let a = map.insert(1);
        let b = map.insert(2);
        let c = map.insert(3);
        map.iter_mut().for_each(|(_, val)| *val *= 2);

        let mut map = map.iter().map(|(i, v)| (i, *v));

        assert_eq!(map.next(), Some((a, 2)));
        assert_eq!(map.next(), Some((b, 4)));
        assert_eq!(map.next(), Some((c, 6)));
        assert_eq!(map.next(), None);
    }

    #[test]
    fn test_keys() {
        let mut map = IndexMap::new();
        let a = map.insert("a");
        let b = map.insert("b");
        let c = map.insert("c");
        map.remove(b);

        let mut keys = map.keys();
        assert_eq!(keys.next(), Some(a));
        assert_eq!(keys.next(), Some(c));
        assert_eq!(keys.next(), None);

        assert_eq!(b, map.insert("b"));

        let mut keys = map.keys();
        assert_eq!(keys.next(), Some(a));
        assert_eq!(keys.next(), Some(b));
        assert_eq!(keys.next(), Some(c));
        assert_eq!(keys.next(), None);
    }

    #[test]
    fn test_values() {
        let mut map = IndexMap::new();
        map.insert("a");
        let b = map.insert("b");
        map.insert("c");
        map.remove(b);
        let mut iter = map.values().map(|v| *v);
        assert_eq!(iter.next(), Some("a"));
        assert_eq!(iter.next(), Some("c"));
        assert_eq!(iter.next(), None);

        assert_eq!(b, map.insert("b"));
        let mut iter = map.values().map(|v| *v);
        assert_eq!(iter.next(), Some("a"));
        assert_eq!(iter.next(), Some("b"));
        assert_eq!(iter.next(), Some("c"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_values_mut() {
        let mut map = IndexMap::new();
        map.insert(1);
        map.insert(2);
        map.insert(3);
        map.values_mut().for_each(|val| *val *= 2);

        let mut map = map.values().map(|v| *v);

        assert_eq!(map.next(), Some(2));
        assert_eq!(map.next(), Some(4));
        assert_eq!(map.next(), Some(6));
        assert_eq!(map.next(), None);
    }
}
