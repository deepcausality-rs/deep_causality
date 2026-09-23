# deep_causality_file

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
 

[crates-badge]: https://img.shields.io/crates/v/deep_causality_file.svg

[crates-url]: https://crates.io/crates/deep_causality_file

[docs-badge]: https://docs.rs/deep_causality_file/badge.svg

[docs-url]: https://docs.rs/deep_causality_file

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

 

## Introduction

`deep_causality_file` provides file and receiver-data loaders and writers for the DeepCausality project. Every loader is
expressed over the **haft IO monad** ([`deep_causality_haft::IoAction`]): a lazy, composable description of a read that
performs no side effect until `.run()` is called at the edge of the program.

The receiver-data format family is **RINEX GNSS** precise products:

* **SP3**: precise satellite orbits (ECEF positions).
* **`.clk`**: precise satellite clocks (bias samples).

This is the real Galileo / multi-GNSS data behind the chronometric and avionics examples (GM recovery, INS clock holdover
through GNSS blackout). The crate also reads and writes typed numeric tables, typed rows, and snapshots, and reads
sensor traces. The loaders are precision-generic over the scalar `R`, so one ingestion path serves every example and the CFD crate.

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

Because the loaders are `IoAction`s, they compose with the haft monadic combinators (`map`, `and_then`) before any read
happens, which keeps side effects at the program boundary.

## Public API

* Loaders: `read_clock_data`, `read_orbit_data`, `read_gnss_single_satellite`, `read_table`, `read_rows`,
  `read_sensor_trace`, `DataManager`, `ReadClockData`, `ReadOrbitData`, `ReadTable`, `ReadRows`, `ReadSensorTrace`.
* Writers: `write_table`, `write_rows`, `WriteTable`, `WriteRows`.
* Snapshots: `save_snapshot`, `load_snapshot`, `force_load_snapshot`, `SaveSnapshot`, `LoadSnapshot`,
  `ForceLoadSnapshot`, `SnapshotPackage`, `SnapshotSection`, `SnapshotTier`, `ScalarTypeTag`, `fingerprint64`, `fnv1a64`.
* Types: `ClockData`, `OrbitData`, `GnssDataResult`, `SatId`, `NumericTable`, `TableColumn`, `SensorChannel`,
  `SensorTraceSet`.
* Traits: `BitCodec`, `FromTableRow`, `TableRow`, `TableScalar`.
* Errors: `ConversionError`, `DataLoadingError`.

## Install

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_file = "0.1"
```

## License

This project is licensed under the [MIT license](LICENSE).
