[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# 🏁 Data structures 🏁

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Audit][audit-url]
![Clippy][clippy-url]
![Tests][test-url]

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/dcl_data_structures

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/dcl_data_structures/latest/dcl_data_structures/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[audit-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/audit.yml/badge.svg

[clippy-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/rust-clippy.yml/badge.svg

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

GridArray and sliding window implementation used in [DeepCausality](https://github.com/deepcausality-rs/deep_causality).

The sliding window implementation over-allocates to trade space (memory) for time complexity by delaying the rewind
operation when hitting the end of the underlying data structure.
Specifically, a sliding window of size N can hold, without any array copy, approximately C-1 elements,
where C is the total capacity defined as NxM with N as the window size and M as a multiple.
This crate has two implementations, one over vector and the second over a const generic array. The const generic
implementation is significantly faster than the vector-based version.

ArrayGrid is an abstraction over scalars, vectors, and low dimensional matrices similar to a tensor.
In contrast to a tensor, an ArrayGrid is limited to low dimensions (1 to 4), only allowing a scalar,
vector, or matrix type. Still, all of them are represented as a static fixed-size const generic array.
Fixed-sized arrays allow for several compiler optimizations, including a cache-aligned data layout and the removal of
runtime array boundary checks because all structural parameters are known upfront, providing a significant performance
boost over tensors.

## 🤔 Why?

1) Zero dependencies.
2) Zero cost abstraction.
3) Zero unsafe. 100% safe and fast Rust.

## 🚀 Install

Just run:

```bash
cargo add dcl_data_structures
```

Alternatively, add the following to your Cargo.toml

```toml
dcl_data_structures = "0.4.2"
```

## 📚 Docs

* [API Docs](https://docs.rs/dcl_data_structures/0.4.3/dcl_data_structures/)
* [ArrayGrid Design & Details](docs/ArrayGrid.md)
* [SlidingWindow Summary](docs/SlidingWindow.md)

## ⭐ Usage

See:

* [Benchmark](benches/benchmarks)
* [Examples](examples)
* [Test](tests)

### ArrayGrid

Important details:

* All const generic parameters are requires regardless of which ArrayType you are using
* To change the ArrayGrid type, just change the enum and your good.
* There are no array bounds checks past compilation, so its your job to ensure PointIndex does not exceed the Array
  boundaries.

```rust
use dcl_data_structures::prelude::{ArrayGrid, ArrayType, PointIndex};

// Consts dimensions requires for const generic paramaters
// Use these to check whether your PointIndex stays within the Array boundaries.
const WIDTH: usize = 5;
const HEIGHT: usize = 5;
const DEPTH: usize = 5;
const TIME: usize = 5;

pub fn main(){
    // Make a simple 1D Array of type usize
    let array_type = ArrayType::Array1D;
    let ag: ArrayGrid<usize, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    // Create a 1D index
    let p = PointIndex::new1d(1);

    // Store a usize with the point index
    ag.set(p, 42);

    // Get the usize for the point index
    let res = ag.get(p);
    assert_eq!(res, 42);
    
    // Make a custom struct 
    // ArrayGrid requires Copy + Default to store MyStuct
    #[derive(Debug, Default, Copy, Clone)]
    struct MyStruct{
        number: usize,
        mod_five: bool,
    }
    
    // Make a 4D array aka matrix over x,y,z that stores My struct
    // Notice, only the ArrayType changes to do that. 
    let array_type = ArrayType::Array4D;
    let ag: ArrayGrid<MyStruct, WIDTH, HEIGHT, DEPTH, TIME> = ArrayGrid::new(array_type);

    // Create a new 4D point index where only time varies
    let idx_t0 = PointIndex::new4d(1, 1, 1, 0);
    let idx_t1 = PointIndex::new4d(1, 1, 1, 1);
    let idx_t2 = PointIndex::new4d(1, 1, 1, 2);

    // Create some data for each index 
    let my_struct_t0 = MyStruct{ number: 23, mod_five: false };
    let my_struct_t1 = MyStruct{ number: 24, mod_five: false };
    let my_struct_t2 = MyStruct{ number: 25, mod_five: true };

    // Store data
    ag.set(idx_t0, my_struct_t0);
    ag.set(idx_t1, my_struct_t1);
    ag.set(idx_t2, my_struct_t2);

    // Get data at t2
    let res = ag.get(idx_t2);
    
    // Verify results
    let exp_number = 25;
    assert_eq!(res.number, exp_number);
    let exp_mod = true;
    assert_eq!(res.mod_five, exp_mod);
}
```

### SlidingWindow

Important details:

* ArrayStorage and VectorStorage have different signatures because only ArrayStorage requires const generics
* Size refers to the maximum number of elements the sliding window can store.
* Capacity refers to the maximum number of elements before a rewind occurs.

```rust
use dcl_data_structures::prelude::{ArrayStorage, SlidingWindow,sliding_window};

// Size refers to the maximum number of elements the sliding window can store.
const SIZE: usize = 4;
// Capacity refers to the maximum number of elements before a rewind occurs.
// Note, CAPACITY > SIZE and capacity should be a multiple of size.
// For example, size 4 should be stored 300 times before rewind;
// 4 * 300 = 1200
const CAPACITY: usize = 1200;

// Util function that helps with type inference.
fn get_sliding_window() -> SlidingWindow<ArrayStorage<Data, SIZE, CAPACITY>, Data> {
    sliding_window::new_with_array_storage()
}

pub fn main(){
    let mut window = get_sliding_window();
    assert_eq!(window.size(), SIZE);

    // Filled means, the window holds 4 elements. 
    assert!(!window.filled());

    // If you try to access an element before the window id filled, you get an error.
    let res = window.first();
    assert!(res.is_err());

    let res = window.last();
    assert!(res.is_err());

    // Add some data
    window.push(Data { dats: 3 });
    window.push(Data { dats: 2 });
    window.push(Data { dats: 1 });
    window.push(Data { dats: 0 });
    assert!(window.filled());

    // Now we can access elements of the window
    // Last element added was 0
    let res = window.last();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 0);

    // First (oldest) element added was 3
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 3);

    // When we add more data after the window filled,
    // the "last" element refers to the last added
    // and the oldest element will be dropped.
    let d = Data { dats: 42 };
    window.push(d);

    let res = window.last();
    assert!(res.is_ok());

    let data = res.unwrap();
    assert_eq!(data.dats, 42);

    // Because 42 was added at the front,
    // 3 was dropped at the end therefore
    // the oldest element is now 2
    let res = window.first();
    assert!(res.is_ok());
    let data = res.unwrap();
    assert_eq!(data.dats, 2);
}
```

## 🙏 Credits

The project took inspiration from:

* [sliding_features](https://crates.io/crates/sliding_features)
* [sliding-window-aggregation](https://crates.io/crates/sliding-window-aggregation)
* [sliding_window_alt](https://crates.io/crates/sliding_window_alt)
* [sliding_windows](https://crates.io/crates/sliding_windows)

## 👨‍💻👩‍💻 Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## 📜 Licence

This project is licensed under the [MIT license](LICENSE).

## 👮️ Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## 💻 Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC