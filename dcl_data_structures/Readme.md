# 💡 Data structures for DeepCausality

Web: https://deepcausality.com/about/

GridArray and sliding window implementation used in [DeepCausality](https://github.com/deepcausality-rs/deep_causality).

The sliding window implementation over-allocates to trade space (memory) for time complexity by delaying the rewind
operation when hitting the end of the underlying data structure.
Specifically, a sliding window of size N can hold, without any array copy, approximately C-1, where C is the total
capacity defined as NxM with M as a multiple.
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

**SlidingWindow**

I noticed three minor issues when looking for existing sliding windows in Rust.
First, either vector was used with a particular performance penalty, or some unsafe code
was used for maximum performance. However, I was looking for a fast yet safe implementation.

Second, a sliding window usually performs better when over-allocated, meaning, for size n, you
allocate k*n with a constant k to delay the index shift when hitting the end of the underlying data structure.
In that sense, you trade memory for better performance. When I looked at existing sliding windows,this feature was not
implemented.

Third, some crates lacked documentation, others seemed abandoned, and so I couldn't find a default implementation to
use; therefore, I wrote the SlidingWindow.

**ArrayGrid**

ArrayGrid became necessary in a trait that has a signature requiring a single type that could represent
multiple shapes, from scalar to a 4D Matrix. Conventionally, a tensor would be a good fit, but since
the signature is part of the hot path, performance became a consideration. After seeing the significant
performance boost resulting from the const generic array implementation of the sliding window, it was time to
take the idea one step further. Therefore, I wrote the ArrayGrid, a single unified type representing
only low dimensional structures (Scalar, Vector, Matrix) as a static const generic array for best performance.

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
* [Test](tests)

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