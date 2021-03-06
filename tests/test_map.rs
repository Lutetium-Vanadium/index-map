// Tests taken from hashbrown test_map

use index_map::IndexMap;
use std::cell::RefCell;

type IM<T> = IndexMap<T>;

#[test]
fn test_zero_capacity() {
    type IM = IndexMap<i32>;
    let m = IM::new();
    assert_eq!(m.capacity(), 0);

    let m = IM::default();
    assert_eq!(m.capacity(), 0);

    let m = IM::with_capacity(0);
    assert_eq!(m.capacity(), 0);

    let mut m = IM::new();
    m.insert(1);
    m.insert(2);
    m.remove(0);
    m.remove(1);
    m.shrink_to_fit();
    assert_eq!(m.capacity(), 0);

    let mut m = IM::new();
    m.reserve(0);
    assert_eq!(m.capacity(), 0);
}

#[test]
fn test_create_capacity_zero() {
    let mut m = IM::with_capacity(0);

    assert_eq!(m.insert(1), 0);

    assert!(m.contains_key(0));
    assert!(!m.contains_key(1));
}

#[test]
fn test_insert() {
    let mut m = IM::new();
    assert_eq!(m.len(), 0);
    assert_eq!(m.insert(2), 0);
    assert_eq!(m.len(), 1);
    assert_eq!(m.insert(4), 1);
    assert_eq!(m.len(), 2);
    assert_eq!(*m.get(0).unwrap(), 2);
    assert_eq!(*m.get(1).unwrap(), 4);
}

#[test]
fn test_clone() {
    let mut m = IM::new();
    assert_eq!(m.len(), 0);
    assert_eq!(m.insert(2), 0);
    assert_eq!(m.len(), 1);
    assert_eq!(m.insert(4), 1);
    assert_eq!(m.len(), 2);
    let m2 = m.clone();
    assert_eq!(*m2.get(0).unwrap(), 2);
    assert_eq!(*m2.get(1).unwrap(), 4);
    assert_eq!(m2.len(), 2);
}

thread_local! { static DROP_VECTOR: RefCell<Vec<i32>> = RefCell::new(Vec::new()) }

#[derive(Hash, PartialEq, Eq)]
struct Droppable {
    k: usize,
}

impl Droppable {
    fn new(k: usize) -> Droppable {
        DROP_VECTOR.with(|slot| {
            slot.borrow_mut()[k] += 1;
        });

        Droppable { k }
    }
}

impl Drop for Droppable {
    fn drop(&mut self) {
        DROP_VECTOR.with(|slot| {
            slot.borrow_mut()[self.k] -= 1;
        });
    }
}

impl Clone for Droppable {
    fn clone(&self) -> Self {
        Droppable::new(self.k)
    }
}

#[test]
fn test_drops() {
    DROP_VECTOR.with(|slot| {
        *slot.borrow_mut() = vec![0; 100];
    });

    {
        let mut m = IM::new();

        DROP_VECTOR.with(|v| {
            for i in 0..100 {
                assert_eq!(v.borrow()[i], 0);
            }
        });

        for i in 0..100 {
            let d = Droppable::new(i);
            m.insert(d);
        }

        DROP_VECTOR.with(|v| {
            for i in 0..100 {
                assert_eq!(v.borrow()[i], 1);
            }
        });

        for i in 0..50 {
            let v = m.remove(i);

            assert!(v.is_some());

            DROP_VECTOR.with(|v| {
                assert_eq!(v.borrow()[i], 1);
            });
        }

        DROP_VECTOR.with(|v| {
            for i in 0..50 {
                assert_eq!(v.borrow()[i], 0);
            }

            for i in 50..100 {
                assert_eq!(v.borrow()[i], 1);
            }
        });
    }

    DROP_VECTOR.with(|v| {
        for i in 0..100 {
            assert_eq!(v.borrow()[i], 0);
        }
    });
}

#[test]
fn test_into_iter_drops() {
    DROP_VECTOR.with(|v| {
        *v.borrow_mut() = vec![0; 100];
    });

    let m = {
        let mut m = IM::new();

        DROP_VECTOR.with(|v| {
            for i in 0..100 {
                assert_eq!(v.borrow()[i], 0);
            }
        });

        for i in 0..100 {
            let d = Droppable::new(i);
            m.insert(d);
        }

        DROP_VECTOR.with(|v| {
            for i in 0..100 {
                assert_eq!(v.borrow()[i], 1);
            }
        });

        m
    };

    // By the way, ensure that cloning doesn't screw up the dropping.
    drop(m.clone());

    {
        let mut half = m.into_iter().take(50);

        DROP_VECTOR.with(|v| {
            for i in 0..100 {
                assert_eq!(v.borrow()[i], 1);
            }
        });

        for _ in half.by_ref() {}

        DROP_VECTOR.with(|v| {
            let n = (0..100).filter(|&i| v.borrow()[i] == 1).count();

            assert_eq!(n, 50);
        });
    };

    DROP_VECTOR.with(|v| {
        for i in 0..100 {
            assert_eq!(v.borrow()[i], 0);
        }
    });
}

#[test]
fn test_empty_remove() {
    let mut m: IM<bool> = IM::new();
    assert_eq!(m.remove(0), None);
}

#[test]
fn test_empty_iter() {
    let mut m: IM<bool> = IM::new();
    assert_eq!(m.drain().next(), None);
    assert_eq!(m.keys().next(), None);
    assert_eq!(m.values().next(), None);
    assert_eq!(m.values_mut().next(), None);
    assert_eq!(m.iter().next(), None);
    assert_eq!(m.iter_mut().next(), None);
    assert_eq!(m.len(), 0);
    assert!(m.is_empty());
    assert_eq!(m.into_iter().next(), None);
}

#[test]
fn test_lots_of_insertions() {
    let mut m = IM::new();

    // Try this a few times to make sure we never screw up the indexmap's
    // internal state.
    for _ in 0..10 {
        assert!(m.is_empty());

        for i in 0..1000 {
            assert_eq!(m.insert(i), i);

            for j in 0..=i {
                let r = m.get(j);
                assert_eq!(r, Some(&j));
            }

            for j in i + 1..1000 {
                let r = m.get(j);
                assert_eq!(r, None);
            }
        }

        for i in 1000..2000 {
            assert!(!m.contains_key(i));
        }

        // remove forwards
        for i in 0..1000 {
            assert!(m.remove(i).is_some());

            for j in 0..=i {
                assert!(!m.contains_key(j));
            }

            for j in i + 1..1000 {
                assert!(m.contains_key(j));
            }
        }

        for i in 0..1000 {
            assert!(!m.contains_key(i));
        }

        // removed it forwards, which means the last thing to be remove is 999, which means this
        // will be the first to be given out.
        for i in (0..1000).rev() {
            assert_eq!(m.insert(i), i);
        }

        // remove backwards
        for i in (0..1000).rev() {
            assert!(m.remove(i).is_some());

            for j in i..1000 {
                assert!(!m.contains_key(j));
            }

            for j in 0..i {
                assert!(m.contains_key(j));
            }
        }
    }
}

#[test]
fn test_find_mut() {
    let mut m = IM::new();
    assert_eq!(m.insert(12), 0);
    assert_eq!(m.insert(8), 1);
    assert_eq!(m.insert(14), 2);
    let new = 100;
    *m.get_mut(2).unwrap() = new;
    assert_eq!(m.get(2), Some(&new));
}

#[test]
fn test_is_empty() {
    let mut m = IM::with_capacity(4);
    assert_eq!(m.insert(2), 0);
    assert!(!m.is_empty());
    assert!(m.remove(0).is_some());
    assert!(m.is_empty());
}

#[test]
fn test_remove() {
    let mut m = IM::new();
    m.insert(2);
    assert_eq!(m.remove(0), Some(2));
    assert_eq!(m.remove(0), None);
}

#[test]
fn test_remove_entry() {
    let mut m = IM::new();
    m.insert(2);
    assert_eq!(m.remove_entry(0), Some((0, 2)));
    assert_eq!(m.remove(0), None);
}

#[test]
fn test_iterate() {
    let mut m = IM::with_capacity(4);
    for i in 0..32 {
        assert_eq!(m.insert(i * 2), i);
    }
    assert_eq!(m.len(), 32);

    let mut observed: u32 = 0;

    for (k, v) in &m {
        assert_eq!(*v, k * 2);
        observed |= 1 << k;
    }
    assert_eq!(observed, 0xFFFF_FFFF);
}

#[test]
fn test_keys() {
    let mut map = IM::new();
    map.insert('a');
    map.insert('b');
    map.insert('c');
    let keys: Vec<_> = map.keys().collect();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&0));
    assert!(keys.contains(&1));
    assert!(keys.contains(&2));
}

#[test]
fn test_values() {
    let mut map = IM::new();
    map.insert('a');
    map.insert('b');
    map.insert('c');

    let values: Vec<_> = map.values().cloned().collect();
    assert_eq!(values.len(), 3);
    assert!(values.contains(&'a'));
    assert!(values.contains(&'b'));
    assert!(values.contains(&'c'));
}

#[test]
fn test_values_mut() {
    let mut map = IM::new();
    map.insert(1);
    map.insert(2);
    map.insert(3);

    for value in map.values_mut() {
        *value = (*value) * 2
    }
    let values: Vec<_> = map.values().cloned().collect();
    assert_eq!(values.len(), 3);
    assert!(values.contains(&2));
    assert!(values.contains(&4));
    assert!(values.contains(&6));
}

#[test]
fn test_find() {
    let mut m = IM::new();
    assert!(m.get(0).is_none());
    m.insert(2);
    assert_eq!(*m.get(0).unwrap(), 2);
}

#[test]
fn test_eq() {
    let mut m1 = IM::new();
    m1.insert(2);
    m1.insert(3);
    m1.insert(4);

    let mut m2 = IM::new();
    m2.insert(2);
    m2.insert(6);
    m2.insert(4);

    assert!(m1 != m2);

    m2.remove(1);
    m2.insert(3);

    assert_eq!(m1, m2);
}

#[test]
fn test_show() {
    let mut map = IM::new();
    let empty: IM<i32> = IM::new();

    map.insert(2);
    map.insert(4);

    let map_str = format!("{:?}", map);

    assert!(map_str == "{0: 2, 1: 4}");
    assert_eq!(format!("{:?}", empty), "{}");
}

#[test]
fn test_reserve_shrink_to_fit() {
    let mut m = IM::new();
    m.insert(0);
    m.remove(0);
    assert!(m.capacity() >= m.len());
    for i in 0..128 {
        m.insert(i);
    }
    m.reserve(256);

    let usable_cap = m.capacity();
    for i in 128..(128 + 256) {
        m.insert(i);
        assert_eq!(m.capacity(), usable_cap);
    }

    for i in 100..(128 + 256) {
        assert_eq!(m.remove(i), Some(i));
    }
    m.shrink_to_fit();

    assert_eq!(m.len(), 100);
    assert!(!m.is_empty());
    assert!(m.capacity() >= m.len());

    for i in 0..100 {
        assert_eq!(m.remove(i), Some(i));
    }
    m.shrink_to_fit();
    m.insert(0);

    assert_eq!(m.len(), 1);
    assert!(m.capacity() >= m.len());
    assert_eq!(m.remove(0), Some(0));
}

#[test]
fn test_size_hint() {
    let mut map = IM::new();
    for i in 0..6 {
        map.insert(i);
    }

    let mut iter = map.iter();

    for _ in iter.by_ref().take(3) {}

    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn test_iter_len() {
    let mut map = IM::new();
    for i in 0..6 {
        map.insert(i);
    }

    let mut iter = map.iter();
    for _ in iter.by_ref().take(3) {}

    assert_eq!(iter.len(), 3);
}

#[test]
fn test_mut_size_hint() {
    let mut map = IM::new();
    for i in 0..6 {
        map.insert(i);
    }

    let mut iter = map.iter_mut();

    for _ in iter.by_ref().take(3) {}

    assert_eq!(iter.size_hint(), (3, Some(3)));
}

#[test]
fn test_iter_mut_len() {
    let mut map = IM::new();
    for i in 0..6 {
        map.insert(i);
    }

    let mut iter = map.iter_mut();

    for _ in iter.by_ref().take(3) {}

    assert_eq!(iter.len(), 3);
}

#[test]
fn test_index() {
    let mut map = IM::new();

    map.insert(2);
    map.insert(1);
    map.insert(4);

    assert_eq!(map[2], 4);
}

#[test]
#[should_panic]
fn test_index_nonexistent() {
    let mut map = IM::new();

    map.insert(2);
    map.insert(1);
    map.insert(4);

    map[4];
}

#[test]
fn test_capacity_not_less_than_len() {
    let mut a = IM::new();

    for _ in 0..116 {
        a.insert(0);
    }

    assert!(a.capacity() > a.len());

    let free = a.capacity() - a.len();
    for _ in 0..free {
        a.insert(0);
    }

    assert_eq!(a.len(), a.capacity());

    // Insert at capacity should cause allocation.
    a.insert(0);
    assert!(a.capacity() > a.len());
}

#[test]
fn test_retain() {
    let mut map = IM::new();
    for i in 0..100 {
        map.insert(i * 10);
    }

    map.retain(|k, _| k % 2 == 0);
    assert_eq!(map.len(), 50);
    assert_eq!(map[2], 20);
    assert_eq!(map[4], 40);
    assert_eq!(map[6], 60);
}
