# 🏁 Data structures for DeepCausality 🏁

Web: https://deepcausality.com/about/

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

1) Zero dependencies. Not quite. See below.
2) Zero cost abstraction.
3) 100% safe Rust

Macros had to move to a separate crate because somehow these can't reside in the same crate using them.


## 🚀 Install

Just run:

```toml
cargo add dcl_data_structures
```

Alternatively, add the following to your Cargo.toml

```toml
dcl_data_structures = "0.4.2"
```
## 📚 Docs

* [ArrayGrid](docs/ArrayGrid.md)
* [SlidingWindow](docs/SlidingWindow.md)

## ⭐ Usage

See:
* [Benchmark](benches/benchmarks)
* [Examples](examples)
* [Test](tests)

Important details:
* All const generic parameters are requires regardless of which ArrayType you are using
* To change the ArrayGrid type, just change the enum and your good.
* There are no array bounds checks past compilation, so its your job to ensure PointIndex does not exceed the Array boundaries.

### ArrayGrid 

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

## 💻 Author

* Marvin Hansen, [Emet-Labs](https://emet-labs.com/).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC