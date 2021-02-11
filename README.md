# Index Map

![build](https://github.com/Lutetium-Vanadium/index-map/workflows/Tests/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/hashbrown.svg)](https://crates.io/crates/index-map)
[![Crates.io](https://img.shields.io/crates/l/hashbrown.svg)](./LICENSE)
[![Documentation](https://docs.rs/index-map/badge.svg)](https://docs.rs/index-map)

A map with automatically generated `usize`s as keys. It supports
[_most_](#method-exclusions) of the methods present on the standard library
[`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html).

If you need a generic key-value pair, with user defined keys, this is
not the crate for you. It is only meant to be used for things that
require automatically generated keys.

## Features

- It automatically generates (and recycles) unique indices as keys,
- Compatible with `#[no_std]` but requires an allocator.
- Doesn't allocate until first value is inserted.
- Values are stored in contiguous memory locations.
- 0 unsafe blocks

## Performance

Due to its simplicity nature, it performs better than `HashMap`.

| name         | IndexMap [ns/iter] | HashMap [ns/iter] | diff [ns/iter] | diff %  | speedup |
| ------------ | ------------------ | ----------------- | -------------- | ------- | ------- |
| insert       | 40,057             | 75,381            | -35,324        | -46.86% | 1.88x   |
| grow_insert  | 12,553             | 100,530           | -87,977        | -87.51% | 8.01x   |
| insert_erase | 9,572              | 12,370            | -2,798         | -22.62% | 1.29x   |
| lookup       | 1,491              | 24,253            | -22,762        | -93.85% | 16.27x  |
| lookup_fail  | 1.289              | 2.092             | -0.8           | -38.38% | 1.62x   |
| bench_iter   | 1,812              | 1,888             | -76            | -4.03%  | 1.04x   |
| clone_small  | 120.30             | 163.70            | -43.4          | -26.51% | 1.36x   |
| clone_large  | 9,635              | 13,041            | -34,06         | -26.12% | 1.35x   |

The benchmarks performed here are the same as present in the hashbrown
repo: [bench.rs](https://github.com/rust-lang/hashbrown/blob/master/benches/bench.rs)

## Usage

Anything than needs to have some unique generated id associated with it
is the perfect use case for this library.

An example could be maintaining a process table.

```rust
use index_map::IndexMap;

let mut process_table = IndexMap::new();

// Create some processes
// Unlike HashMap, insert only takes a value, and returns the key.
let vim = process_table.insert("vim".to_string());
//  ^^^----------------------------------------------------------.
let cargo = process_table.insert("cargo".to_string()); //        |
//  ^^^^^--------------------------------------------------------.
let rls = process_table.insert("rust-analyser".to_string()); //  |
//  ^^^----------------------------------------------------------|
//                                                               |
//  Unique numbers representing each process  <------------------'

// Check for a specific one.
if !process_table.contains_key(6) {
    println!("Invalid PID 6}");
}

// cargo finished running, remove it
process_table.remove(cargo);

// Look up the values associated with some keys.
let to_find = [2, 4];
for &pid in &to_find {
    match process_table.get(pid) {
        Some(process) => println!("{}: {}", pid, process),
        None => println!("{} not found", pid)
    }
}

// Look up the value for a key (will panic if the key is not found).
println!("PID 0 process name: {}", process_table[0]);

// Iterate over everything.
for (pid, process) in &process_table {
    println!("{}: \"{}\"", pid, process);
}
```

## Method Exclusions

- nightly only/unstable

- hasher based functions - the internal structure is based on a `Vec`
  and so hashes are not needed.

- [`entry()`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry) -
  Although all the methods for `OccupiedEntry` are possible, fundamentally,
  keys are automatically generated, and so the methods based on
  `VacantEntry` are not possible.

- [`Extend`](https://doc.rust-lang.org/std/iter/trait.Extend.html) &
  [`FromIterator`](https://doc.rust-lang.org/std/iter/trait.FromIterator.html) -
  It can't take key-value pairs like `HashMap` because of the reason
  mentioned above, and it can't return the keys for values inserted.

## Contribution

If you find any bugs or issues, or you want some feature added, feel
free to open an [issue](https://github.com/Lutetium-Vanadium/index-map/issues/new)
or a [pull request](https://github.com/Lutetium-Vanadium/index-map/compare).
