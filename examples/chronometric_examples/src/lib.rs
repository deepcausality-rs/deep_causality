/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Chronometric examples support. The GNSS data **types** (`ClockData`, `OrbitData`, `SatId`,
//! `GnssDataResult`) and the RINEX **loaders** now live in the shared `deep_causality_file` crate and
//! are re-exported here so the examples (and `gm_recovery`) keep one import path. The
//! chronometric-specific processing — Lagrange space-time alignment, the MAD outlier filter, the
//! gravity/GM types — stays local.

pub mod data_manager;
pub mod proces_utils;
pub(crate) mod types;

// Re-export the shared GNSS ingestion surface from `deep_causality_file`.
pub use deep_causality_file::{ClockData, ConversionError, GnssDataResult, OrbitData, SatId};
