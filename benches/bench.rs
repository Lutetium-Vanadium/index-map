use criterion::{black_box, criterion_group, criterion_main, Criterion};
use index_map::IndexMap;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};

const SIZE: usize = 1000;
const KEY_DIST: Range<usize> = 0..SIZE;

// Just an arbitrary side effect to make the maps not shortcircuit to the non-dropping path
// when dropping maps/entries (most real world usages likely have drop in the key or value)
static SIDE_EFFECT: AtomicUsize = AtomicUsize::new(0);

// Setting this to true benches the standard library HashMap, with the same tests.
const BENCH_HASHMAP: bool = false;

#[derive(Clone)]
struct DropType(usize);
impl Drop for DropType {
    fn drop(&mut self) {
        SIDE_EFFECT.fetch_add(self.0, Ordering::SeqCst);
    }
}

fn insert(c: &mut Criterion) {
    let mut m = IndexMap::with_capacity(SIZE);
    c.bench_function("insert", |b| {
        b.iter(|| {
            m.clear();
            for i in KEY_DIST {
                m.insert((DropType(i), [i; 20]));
            }
            black_box(&mut m);
        });
    });

    if BENCH_HASHMAP {
        let mut m = HashMap::with_capacity(SIZE);
        c.bench_function("hash_map-insert", |b| {
            b.iter(|| {
                m.clear();
                for i in KEY_DIST {
                    m.insert(i, (DropType(i), [i; 20]));
                }
                black_box(&mut m);
            });
        });
    }
}

fn grow_insert(c: &mut Criterion) {
    c.bench_function("grow_insert", |b| {
        b.iter(|| {
            let mut m = IndexMap::default();
            for i in KEY_DIST.take(SIZE) {
                m.insert(DropType(i));
            }
            black_box(&mut m);
        });
    });

    if BENCH_HASHMAP {
        c.bench_function("hash_map-grow_insert", |b| {
            b.iter(|| {
                let mut m = HashMap::new();
                for i in KEY_DIST.take(SIZE) {
                    m.insert(i, DropType(i));
                }
                black_box(&mut m);
            });
        });
    }
}

fn insert_erase(c: &mut Criterion) {
    let mut base = IndexMap::default();
    for i in KEY_DIST.take(SIZE) {
        base.insert(DropType(i));
    }
    let skip = KEY_DIST.skip(SIZE);
    c.bench_function("insert_erase", |b| {
        b.iter(|| {
            let mut m = base.clone();
            let mut add_iter = skip.clone();
            let mut remove_iter = KEY_DIST;
            // While keeping the size constant,
            // replace the first keydist with the second.
            for (add, remove) in (&mut add_iter).zip(&mut remove_iter).take(SIZE) {
                m.insert(DropType(add));
                black_box(m.remove(remove));
            }
            black_box(m);
        });
    });

    if BENCH_HASHMAP {
        let mut base = HashMap::new();
        for i in KEY_DIST.take(SIZE) {
            base.insert(i, DropType(i));
        }
        let skip = KEY_DIST.skip(SIZE);
        c.bench_function("hash_map-insert_erase", |b| {
            b.iter(|| {
                let mut m = base.clone();
                let mut add_iter = skip.clone();
                let mut remove_iter = KEY_DIST;
                // While keeping the size constant,
                // replace the first keydist with the second.
                for (add, remove) in (&mut add_iter).zip(&mut remove_iter).take(SIZE) {
                    m.insert(add, DropType(add));
                    black_box(m.remove(&remove));
                }
                black_box(m);
            });
        });
    }
}

fn lookup(c: &mut Criterion) {
    let mut m = IndexMap::default();
    for i in KEY_DIST.take(SIZE) {
        m.insert(DropType(i));
    }

    c.bench_function("lookup", |b| {
        b.iter(|| {
            for i in KEY_DIST.take(SIZE) {
                black_box(m.get(i));
            }
        });
    });

    if BENCH_HASHMAP {
        let mut m = HashMap::new();
        for i in KEY_DIST.take(SIZE) {
            m.insert(i, DropType(i));
        }

        c.bench_function("hash_map-lookup", |b| {
            b.iter(|| {
                for i in KEY_DIST.take(SIZE) {
                    black_box(m.get(&i));
                }
            });
        });
    }
}

fn lookup_fail(c: &mut Criterion) {
    let mut m = IndexMap::default();
    let mut iter = KEY_DIST;
    for i in (&mut iter).take(SIZE) {
        m.insert(DropType(i));
    }

    c.bench_function("lookup_fail", |b| {
        b.iter(|| {
            for i in (&mut iter).take(SIZE) {
                black_box(m.get(i));
            }
        });
    });

    if BENCH_HASHMAP {
        let mut m = HashMap::new();
        let mut iter = KEY_DIST;
        for i in (&mut iter).take(SIZE) {
            m.insert(i, DropType(i));
        }

        c.bench_function("hash_map-lookup_fail", |b| {
            b.iter(|| {
                for i in (&mut iter).take(SIZE) {
                    black_box(m.get(&i));
                }
            });
        });
    }
}

fn bench_iter(c: &mut Criterion) {
    let mut m = IndexMap::default();
    for i in KEY_DIST.take(SIZE) {
        m.insert(DropType(i));
    }

    c.bench_function("bench_iter", |b| {
        b.iter(|| {
            for i in &m {
                black_box(i);
            }
        });
    });

    if BENCH_HASHMAP {
        let mut m = HashMap::new();
        for i in KEY_DIST.take(SIZE) {
            m.insert(i, DropType(i));
        }

        c.bench_function("hash_map-bench_iter", |b| {
            b.iter(|| {
                for i in &m {
                    black_box(i);
                }
            });
        });
    }
}

fn clone_small(c: &mut Criterion) {
    let mut m = IndexMap::new();
    for i in 0..10 {
        m.insert(DropType(i));
    }

    c.bench_function("clone_small", |b| {
        b.iter(|| {
            black_box(m.clone());
        });
    });

    if BENCH_HASHMAP {
        let mut m = HashMap::new();
        for i in 0..10 {
            m.insert(i, DropType(i));
        }

        c.bench_function("hash_map-clone_small", |b| {
            b.iter(|| {
                black_box(m.clone());
            });
        });
    }
}

fn clone_large(c: &mut Criterion) {
    let mut m = IndexMap::new();
    for i in 0..1000 {
        m.insert(DropType(i));
    }

    c.bench_function("clone_large", |b| {
        b.iter(|| {
            black_box(m.clone());
        });
    });

    if BENCH_HASHMAP {
        let mut m = HashMap::new();
        for i in 0..1000 {
            m.insert(i, DropType(i));
        }

        c.bench_function("hash_map-clone_large", |b| {
            b.iter(|| {
                black_box(m.clone());
            });
        });
    }
}

criterion_group!(
    benches,
    insert,
    grow_insert,
    insert_erase,
    lookup,
    lookup_fail,
    bench_iter,
    clone_small,
    clone_large
);
criterion_main!(benches);
