/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # deep_causality_file
//!
//! File and receiver-data loaders for DeepCausality, expressed over the **haft IO monad**
//! ([`deep_causality_haft::IoAction`]) — lazy, composable descriptions of a read that perform no
//! side effect until `.run()` at the program edge.
//!
//! The first supported format family is **RINEX GNSS** precise products: SP3 (satellite orbits) and
//! `.clk` (satellite clocks), the real Galileo/multi-GNSS data behind the chronometric and avionics
//! examples (GM recovery, INS clock-holdover through GNSS blackout). The loaders are precision-generic
//! over the scalar `R` and live behind one reusable crate so every example — and the CFD crate — can
//! consume one ingestion path.
//!
//! ```no_run
//! use deep_causality_file::{read_gnss_single_satellite, ClockData, OrbitData};
//! use deep_causality_haft::IoAction;
//!
//! // A lazy description of two file reads composed with the IO monad; nothing runs yet.
//! let action = read_gnss_single_satellite::<f64>("gbm.clk", "gbm.sp3", "E14");
//! // Perform the read at the edge.
//! let (_clocks, _orbits): (Vec<ClockData<f64>>, Vec<OrbitData<f64>>) = action.run().unwrap();
//! ```

pub mod errors;
pub mod traits;
pub mod types;

pub use errors::conversion_error::ConversionError;
pub use errors::data_loading_error::DataLoadingError;
pub use traits::bit_codec::BitCodec;
pub use traits::table_row::{FromTableRow, TableRow};
pub use traits::table_scalar::TableScalar;
pub use types::clock_types::ClockData;
pub use types::gnss_types::GnssDataResult;
pub use types::loaders::{
    DataManager, ReadClockData, ReadOrbitData, ReadRows, ReadSensorTrace, ReadTable,
    read_clock_data, read_gnss_single_satellite, read_orbit_data, read_rows, read_sensor_trace,
    read_table,
};
pub use types::orbit_types::OrbitData;
pub use types::satelite_types::SatId;
pub use types::snapshot::{
    ForceLoadSnapshot, LoadSnapshot, SaveSnapshot, fingerprint64, fnv1a64, force_load_snapshot,
    load_snapshot, save_snapshot,
};
pub use types::snapshot_types::{ScalarTypeTag, SnapshotPackage, SnapshotSection, SnapshotTier};
pub use types::table_types::{NumericTable, TableColumn};
pub use types::trace_types::{SensorChannel, SensorTraceSet};
pub use types::writers::{WriteRows, WriteTable, write_rows, write_table};
