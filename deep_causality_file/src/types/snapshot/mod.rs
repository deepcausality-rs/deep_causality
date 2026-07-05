/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The snapshot container: format, checksum, and the save / load / force-load IO actions.

pub mod checksum;
pub mod container;
pub mod io;

pub use checksum::{fingerprint64, fnv1a64};
pub use io::{
    ForceLoadSnapshot, LoadSnapshot, SaveSnapshot, force_load_snapshot, load_snapshot,
    save_snapshot,
};
