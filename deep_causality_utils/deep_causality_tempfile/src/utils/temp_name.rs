/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Unique paths for scratch entries under the OS temp directory.

use std::path::PathBuf;

/// Returns `std::env::temp_dir()` joined with `.dct-{pid}-{nanos}-{counter}{suffix}`.
///
/// `pid` is the process id, `nanos` the wall clock in nanoseconds since the Unix epoch, and
/// `counter` a process-wide count incremented on every call. The pid and counter keep names
/// distinct across live processes and within one process; the clock separates a process from an
/// earlier one that had the same pid. The caller validates `suffix`.
pub(crate) fn temp_path(suffix: &str) -> PathBuf {
    let _ = suffix;
    unimplemented!()
}
