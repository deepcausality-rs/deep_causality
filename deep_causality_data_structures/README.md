[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# 🏁 Data structures 🏁

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
 

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality_data_structures

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality_data_structures/latest/deep_causality_data_structures/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

 

This crate provides two data structures used in [DeepCausality](https://github.com/deepcausality-rs/deep_causality):
ArrayGrid and SlidingWindow.

ArrayGrid abstracts over scalars, vectors, and low-dimensional matrices, similar to a tensor but
limited to 1 to 4 dimensions. Every ArrayGrid is a static, fixed-size const generic array.
Because all structural parameters are known at compile time, the compiler can lay the data out
cache-aligned, which makes ArrayGrid faster than a tensor.


The sliding window over-allocates, trading memory for time by delaying the rewind when it reaches
the end of the underlying storage. A sliding window of size N holds approximately C-1 elements
without any array copy, where the capacity C is NxM with N the window size and M a multiple.
The crate implements the window over a vector and over a const generic array; the const generic
version is significantly faster.

## Why?

1) Zero cost abstraction.
2) Zero unsafe.
3) Zero macros.
4) Zero external dependencies.

# Performance:

## ArrayGrid

**Set value:**

| Dimension | Safe Implementation | Unsafe Implementation | Improvement |
|-----------|---------------------|-----------------------|-------------|
| 1D Grid   | 604.71 ps           | 271.38 ps             | 55.1%       |
| 2D Grid   | 581.33 ps           | 417.39 ps             | 28.2%       |
| 3D Grid   | 862.16 ps           | 577.04 ps             | 33.0%       |
| 4D Grid   | 1.137 ns            | 812.62 ps             | 28.5%       |

See the [Performance](README_ArrayGrid.md#performance) section
of the [ArrayGrid document](README_ArrayGrid.md) for details.

## Sliding Window

**Single Push:**

| Implementation      	 | Single Push Time 	 | Notes                                                	 |
|-----------------------|--------------------|--------------------------------------------------------|
| ArrayStorage        	 | ~2.08ns          	 | Optimized for continuous access patterns             	 |
| VectorStorage       	 | ~2.5ns           	 | Good for dynamic sizing                              	 |
| UnsafeVectorStorage 	 | ~2.3ns           	 | Better performance than safe vector                  	 |
| UnsafeArrayStorage  	 | ~1.9ns           	 | Best performance for sequential and batch operations 	 |

**Sequential Operations:**

| Implementation      | Operation Time | Notes                    | 
|---------------------|----------------|--------------------------| 
| UnsafeArrayStorage  | ~550ps         | Best cache utilization   | 
| ArrayStorage        | ~605ps         | Excellent cache locality | 
| UnsafeVectorStorage | ~750ps         | Good for mixed workloads | 
| VectorStorage       | ~850ps         | Most predictable         |

See the [Performance](README_SlidingWindow.md#performance) section
of the [SlidingWindow document](README_SlidingWindow.md) for details.

## Install

Run:

```bash
cargo add deep_causality_data_structures
```

## Docs

* [API Docs](https://docs.rs/deep_causality_data_structures/latest/deep_causality_data_structures)
* [ArrayGrid Summary](README_ArrayGrid.md)
* [CausalTensor Summary](../deep_causality_tensor/README.md)
* [SlidingWindow Summary](README_SlidingWindow)

## Usage

**ArrayGrid:**
* [Design & Details](README_ArrayGrid)
* [Benchmark](benches/benchmarks)
* [Examples](examples/array_grid)
* [Test](tests/grid_type)

* **SlidingWindow:**
* [Design & Details](README_SlidingWindow.md)
* [Benchmark](benches/benchmarks)
* [Examples](examples/window_type)
* [Test](tests/window_type)

## Prior Art

The project took inspiration from:

* [sliding_features](https://crates.io/crates/sliding_features)
* [sliding-window-aggregation](https://crates.io/crates/sliding-window-aggregation)
* [sliding_window_alt](https://crates.io/crates/sliding_window_alt)
* [sliding_windows](https://crates.io/crates/sliding_windows)

## Contribution

Contributions are welcome, especially documentation, example code, and fixes.
If unsure where to start, open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

