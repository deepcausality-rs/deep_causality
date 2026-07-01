/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Data-location helper for the bundled GNSS fixtures, plus a re-export of the shared
//! [`DataManager`](deep_causality_file::DataManager) that performs the RINEX loads.

use std::path::PathBuf;

/// The shared GNSS data-loading facade (loaders live in `deep_causality_file`).
pub use deep_causality_file::DataManager;

/// Absolute path to this crate's bundled GNSS data directory (`data/gnss`).
pub fn get_gnss_data_input_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("data/gnss");
    path
}
