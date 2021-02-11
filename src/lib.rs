#![no_std]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
//! A map with automatically generated `usize`s as keys.
//!
//! # Usage
//!
//! ```
//! use index_map::IndexMap;
//!
//! let mut process_table = IndexMap::new();
//!
//! // Create some processes
//! // Unlike HashMap, insert only takes a value, and returns the key.
//! let vim = process_table.insert("vim".to_string());
//! //  ^^^----------------------------------------------------------.
//! let cargo = process_table.insert("cargo".to_string()); //        |
//! //  ^^^^^--------------------------------------------------------|
//! let rls = process_table.insert("rust-analyser".to_string()); //  |
//! //  ^^^----------------------------------------------------------|
//! //                                                               |
//! //  Unique numbers representing each process  <------------------'
//!
//! // Check for a specific one.
//! if !process_table.contains_key(6) {
//!     println!("Invalid PID 6");
//! }
//!
//! // cargo finished running, remove it
//! process_table.remove(cargo);
//!
//! // Look up the values associated with some keys.
//! let to_find = [2, 4];
//! for &pid in &to_find {
//!     match process_table.get(pid) {
//!         Some(process) => println!("{}: {}", pid, process),
//!         None => println!("{} not found", pid)
//!     }
//! }
//!
//! // Look up the value for a key (will panic if the key is not found).
//! println!("PID 0 process name: {}", process_table[0]);
//!
//! // Iterate over everything.
//! for (pid, process) in &process_table {
//!     println!("{}: \"{}\"", pid, process);
//! }
//! ```
//!
//! # How it works
//! It internally is based on a [`Vec`], where each element either stores a value, or stores the index
//! of the next free element. Since it accommodates for empty elements in between, it can perform
//! O(1)* inserts and O(1) removals from any index. The "free" indices essentially make a singly
//! linked list.
//!
//! \* amortized similar to [`Vec::push()`] (triggers a resize when [`len()`](IndexMap::len) ==
//! [`capacity()`](IndexMap::capacity))
//!
//! Take the following example:
//! > `*` represents an element \
//! >`-` represents no index (end of the linked list) \
//! >`<int>` represents the index of the next free element
//!
//! Assuming there are already 3 elements,
//!
//! ```text
//! Initial State:
//! .---.---.---.
//! | * | * | * | head - None
//! '---'---'---'
//!   0   1   2
//!
//! Op - remove(1)
//! State:
//! .---.---.---.
//! | * | - | * |
//! '---'---'---'
//!       ^-- head [ 1 ]
//!
//! Op - remove(2)
//! State:
//! ```text
//! .---.---.---.
//! | * | - | 1 |
//! '---'---'---'
//!           ^-- head [ 2 ]
//!
//! Op - insert
//! State:
//! .---.---.---.
//! | * | - | * |
//! '---'---'---'
//!       ^-- head [ 1 ]
//! ```

extern crate alloc;

use alloc::vec::Vec;

mod iter;
mod option_index;
pub use iter::{Drain, IntoIter, Iter, IterMut, Keys, Values, ValuesMut};
use option_index::OptionIndex;

/// A map of `usize` to value, which allows efficient O(1) inserts, O(1) indexing and O(1) removal.
///
/// See [crate level documentation](crate) for more information.
#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct IndexMap<T> {
    data: Vec<OptionIndex<T>>,
    head: Option<usize>,
    len: usize,
}

impl<T> IndexMap<T> {
    /// Creates a new `IndexMap`.
    ///
    /// It initially has a capacity of 0, and won't allocate until first inserted into.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    /// let mut map: IndexMap<&str> = IndexMap::new();
    /// ```
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            head: None,
            len: 0,
        }
    }

    /// Creates an empty `IndexMap` with the specified capacity.
    ///
    /// The map will be able to hold at least capacity elements without reallocating. If capacity
    /// is 0, the map will not allocate.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    /// let mut map: IndexMap<&str> = IndexMap::with_capacity(10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            head: None,
            len: 0,
        }
    }

    /// Returns the number of elements map can hold without reallocating.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    /// let mut map: IndexMap<&str> = IndexMap::with_capacity(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Returns the number of elements present in the map.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    /// let mut map = IndexMap::new();
    /// assert_eq!(map.len(), 0);
    /// map.insert("a");
    /// assert_eq!(map.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the map is empty.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    /// let mut map = IndexMap::new();
    /// assert!(map.is_empty());
    /// map.insert("a");
    /// assert!(!map.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the map, dropping all key-value pairs. Keeps the allocated memory for reuse.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    /// let mut map = IndexMap::new();
    ///
    /// map.insert("a");
    /// map.clear();
    ///
    /// assert!(map.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.len = 0;
        self.data.clear()
    }

    /// Reserves capacity for at least additional more elements to be inserted in the `IndexMap`
    /// The collection may reserve more space to avoid frequent reallocations.
    ///
    /// # Panics
    /// Panics if the new capacity exceeds [`isize::MAX`] bytes.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    /// let mut map: IndexMap<&str> = IndexMap::new();
    /// map.reserve(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional)
    }

    /// Shrinks the capacity of the map as much as possible. It will drop down as much as possible
    /// while maintaining the internal rules and possibly leaving some space to keep keys valid.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    /// let mut map = IndexMap::with_capacity(100);
    /// map.insert("a");
    /// map.insert("b");
    /// assert!(map.capacity() >= 100);
    /// map.shrink_to_fit();
    /// assert!(map.capacity() >= 2);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        // This relies on the fact that `||` short-circuits. If `data` is empty, `head` *has* to be
        // None, and so `data.last()` *cannot* be None.
        if self.head.is_none() || self.data.last().unwrap().is_inner() {
            self.data.shrink_to_fit();
            return;
        }

        if self.is_empty() {
            self.head = None;
            self.data.clear();
            self.data.shrink_to_fit();
            return;
        }

        // random default value, the previous check makes sure there are elements, so the if
        // condition has to be triggered.
        let mut last = usize::MAX;

        for (i, v) in self.data.iter().enumerate().rev() {
            if v.is_inner() {
                last = i;
                break;
            }
        }

        assert_ne!(last, usize::MAX);

        let mut head = self.head.unwrap();
        // If head is more than last, it needs to be set in such a way that head points to an
        // index which is not truncated
        //                   ,-- head [ 4 ]   |   Key:
        // .---.---.---.---.---.              |   *     = element
        // | * | - | * | * | 1 |              |   -     = No Index
        // '---'---'---'---'---'              |   <int> = Index
        //               ^-- last [ 3 ]       |
        // Take the above data. After shrinking, it would be erroneous for head to still point
        // to 4, since it will be deleted.
        let mut should_set_head = head > last;

        while let OptionIndex::Index(next) = self.data[head] {
            if next > last {
                // We can't use clone because `T` is not required to be clone, so no bound is
                // added. We can't use `OptionIndex::take`, since we need the index intact for
                // the next loop.
                self.data[head] = match self.data[next] {
                    OptionIndex::Index(i) => OptionIndex::Index(i),
                    OptionIndex::NoIndex => OptionIndex::NoIndex,
                    OptionIndex::Some(_) => {
                        unreachable!("encountered value while walking index list")
                    }
                };
            }

            if should_set_head && head < last {
                self.head = Some(head);
                should_set_head = false;
            }
            head = next;
        }

        // The only index not checked is `head`, replace `self.head` based on it.
        if should_set_head {
            self.head = if head < last { Some(head) } else { None };
        }

        self.data[head] = OptionIndex::NoIndex;

        // Truncate expects length, not the index of last element
        self.data.truncate(last + 1);

        self.data.shrink_to_fit()
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// # Examples
    ///
    /// ```
    /// use index_map::IndexMap;
    ///
    /// let mut map = IndexMap::new();
    /// map.insert("a");
    /// assert_eq!(map.contains_key(0), true);
    /// assert_eq!(map.contains_key(1), false);
    /// ```
    pub fn contains_key(&self, index: usize) -> bool {
        if index >= self.data.len() {
            return false;
        }

        self.data[index].is_inner()
    }

    /// Inserts a value into the map, returning the generated key, for it.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    ///
    /// let mut map = IndexMap::new();
    /// assert_eq!(map.insert("a"), 0);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// let b = map.insert("b");
    /// assert_eq!(map[b], "b");
    /// ```
    pub fn insert(&mut self, value: T) -> usize {
        // The operation can't fail (unless Vec panics internally) since the key is generated by us.
        self.len += 1;

        if let Some(head) = self.head {
            self.head = self.data[head].take().into_index();
            self.data[head] = OptionIndex::Some(value);
            head
        } else {
            self.data.push(OptionIndex::Some(value));
            self.data.len() - 1
        }
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in
    /// the map.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    ///
    /// let mut map = IndexMap::new();
    /// let a = map.insert("a");
    /// assert_eq!(map.remove(a), Some("a"));
    /// assert_eq!(map.remove(a), None);
    /// ```
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if !self.data.get(index)?.is_inner() {
            return None;
        }

        let val = self.data.get_mut(index)?.take().into_inner()?;

        if let Some(head) = self.head {
            self.data[index] = OptionIndex::Index(head);
        } else {
            self.data[index] = OptionIndex::NoIndex;
        }

        self.head = Some(index);
        self.len -= 1;

        Some(val)
    }

    /// Removes a key from the map, returning the key and value if the key was previously in the map.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    ///
    /// let mut map = IndexMap::new();
    /// let a = map.insert("a");
    /// assert_eq!(map.remove_entry(a), Some((0, "a")));
    /// assert_eq!(map.remove(a), None);
    /// ```
    pub fn remove_entry(&mut self, index: usize) -> Option<(usize, T)> {
        Some((index, self.remove(index)?))
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    ///
    /// let mut map = IndexMap::new();
    /// map.insert("a");
    /// assert_eq!(map.get(0), Some(&"a"));
    /// assert_eq!(map.get(1), None);
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)?.as_ref().into_inner()
    }

    /// Returns the key-value pair corresponding to the key.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    ///
    /// let mut map = IndexMap::new();
    /// map.insert("a");
    /// assert_eq!(map.get_key_value(0), Some((0, &"a")));
    /// assert_eq!(map.get_key_value(1), None);
    /// ```
    pub fn get_key_value(&self, index: usize) -> Option<(usize, &T)> {
        Some((index, self.get(index)?))
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    ///
    /// let mut map = IndexMap::new();
    /// let a = map.insert("a");
    /// if let Some(x) = map.get_mut(a) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[a], "b");
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)?.as_mut().into_inner()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` such that `f(k, &mut v)` returns `false`.
    ///
    /// # Examples
    /// ```
    /// use index_map::IndexMap;
    ///
    /// let mut map = IndexMap::new();
    /// for i in 0..6 {
    ///     map.insert(i*2);
    /// }
    /// map.retain(|k, _| k % 2 == 0);
    /// assert_eq!(map.len(), 3);
    /// ```
    pub fn retain<P>(&mut self, mut predicate: P)
    where
        P: FnMut(usize, &mut T) -> bool,
    {
        // Cannot use `self.iter_mut` as we need the pointer to the `OptionIndex` and not the value
        // contained in it.
        for (i, v) in self.data.iter_mut().enumerate() {
            if let OptionIndex::Some(val) = v {
                if !predicate(i, val) {
                    *v = if let Some(head) = self.head {
                        OptionIndex::Index(head)
                    } else {
                        OptionIndex::NoIndex
                    };

                    self.len -= 1;
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
            len: self.len,
        }
    }
}

impl<T> Default for IndexMap<T> {
    /// Creates an empty `IndexMap`, same as calling new.
    fn default() -> Self {
        Self::new()
    }
}

use core::fmt;

impl<T: fmt::Debug> fmt::Debug for IndexMap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

use core::ops::{Index, IndexMut};

impl<T> Index<usize> for IndexMap<T> {
    type Output = T;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    /// Panics if the key is not present in the `IndexMap`.
    fn index(&self, key: usize) -> &T {
        self.get(key).unwrap()
    }
}

impl<T> IndexMut<usize> for IndexMap<T> {
    /// Returns a mutable reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    /// Panics if the key is not present in the `IndexMap`.
    fn index_mut(&mut self, key: usize) -> &mut T {
        self.get_mut(key).unwrap()
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
