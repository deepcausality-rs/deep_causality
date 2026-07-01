# deep_causality_file

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
![Tests][test-url]

[crates-badge]: https://img.shields.io/crates/v/deep_causality_file.svg

[crates-url]: https://crates.io/crates/deep_causality_file

[docs-badge]: https://docs.rs/deep_causality_file/badge.svg

[docs-url]: https://docs.rs/deep_causality_file

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[test-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/run_tests.yml/badge.svg

## Introduction

`deep_causality_file` provides file and receiver-data loaders for the DeepCausality project. Every loader is expressed
over the **haft IO monad** ([`deep_causality_haft::IoAction`]): a lazy, composable description of a read that performs no
side effect until `.run()` is called at the edge of the program.

The first supported format family is **RINEX GNSS** precise products:

* **SP3** — precise satellite orbits (ECEF positions).
* **`.clk`** — precise satellite clocks (bias samples).

This is the real Galileo / multi-GNSS data behind the chronometric and avionics examples (GM recovery, INS clock holdover
through GNSS blackout). The loaders are precision-generic over the scalar `R`, so a single ingestion path serves every
example — and the CFD crate — without duplicating parsing code.

## Loaders

| Function | Returns | Reads |
|----------|---------|-------|
| `read_clock_data::<R>(path, sat)` | `Vec<ClockData<R>>` | one `.clk` file, one satellite |
| `read_orbit_data::<R>(path, sat)` | `Vec<OrbitData<R>>` | one SP3 file, one satellite |
| `read_gnss_single_satellite::<R>(clk, sp3, sat)` | `(Vec<ClockData<R>>, Vec<OrbitData<R>>)` | both files, composed with the IO monad |

Each function returns an `IoAction` (`ReadClockData`, `ReadOrbitData`, or the composed action). Nothing touches the
filesystem until `.run()`.

## Usage

```rust
use deep_causality_file::{read_gnss_single_satellite, ClockData, OrbitData};
use deep_causality_haft::IoAction;

// A lazy description of two file reads composed with the IO monad; nothing runs yet.
let action = read_gnss_single_satellite::<f64>("gbm.clk", "gbm.sp3", "E14");

// Perform the read at the edge.
let (clocks, orbits): (Vec<ClockData<f64>>, Vec<OrbitData<f64>>) = action.run().unwrap();
```

Because the loaders are `IoAction`s, they compose with the rest of the haft monadic combinators (`map`, `and_then`)
before any read happens, keeping side effects at the program boundary.

## Public API

* Loaders: `read_clock_data`, `read_orbit_data`, `read_gnss_single_satellite`, `DataManager`,
  `ReadClockData`, `ReadOrbitData`.
* Types: `ClockData`, `OrbitData`, `GnssDataResult`, `SatId`.
* Errors: `ConversionError`, `DataLoadingError`.

## Install

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_file = "0.1"
```

## License

This project is licensed under the [MIT license](LICENSE).
