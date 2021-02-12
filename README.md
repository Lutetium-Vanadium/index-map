# Index Map

![build](https://github.com/Lutetium-Vanadium/index-map/workflows/Tests/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/index-map.svg)](https://crates.io/crates/index-map)
[![Crates.io](https://img.shields.io/crates/l/index-map.svg)](./LICENSE)
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

Due to its simplicity nature, it performs slightly better than `HashMap`.

| name         | IndexMap [ns/iter] | HashMap [ns/iter] | diff [ns/iter] | diff %  | speedup |
| ------------ | ------------------ | ----------------- | -------------- | ------- | ------- |
| insert       | 38,395             | 52,829            | -14,434        | -27.32% | 1.38x   |
| grow_insert  | 11,970             | 44,681            | -32,711        | -73.21% | 3.73x   |
| insert_erase | 9,429              | 12,563            | -3,134         | -24.95% | 1.33x   |
| lookup       | 1,479              | 4,202             | -2,723         | -64.80% | 2.84x   |
| lookup_fail  | 1.271              | 1.709             | -0.438         | -25.63% | 1.34x   |
| bench_iter   | 1,788              | 2,423             | -635           | -26.21% | 1.36x   |
| clone_small  | 114.49             | 158.66            | -44.17         | -27.84% | 1.39x   |
| clone_large  | 8,693              | 12,517            | -3,824         | -30.55% | 1.44x   |

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
