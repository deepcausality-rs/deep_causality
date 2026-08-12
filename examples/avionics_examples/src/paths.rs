/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Resolves this example package's root directory under both build systems.

use std::path::PathBuf;

/// This package's directory, relative to the workspace root.
const PACKAGE_DIR: &str = "examples/avionics_examples";

/// The directory holding this package's `Cargo.toml`, resolved at run time.
///
/// Under `cargo run` that is `CARGO_MANIFEST_DIR`. Under `bazel run` the compile-time
/// `CARGO_MANIFEST_DIR` names the rustc sandbox, which is gone by the time the binary runs,
/// so the workspace root Bazel exports in `BUILD_WORKSPACE_DIRECTORY` is used instead. Both
/// paths land on the same directory in the source tree, so the examples read their input
/// fixtures and record their output tables in the same place either way.
pub fn manifest_dir() -> PathBuf {
    match std::env::var_os("BUILD_WORKSPACE_DIRECTORY") {
        Some(workspace_root) => PathBuf::from(workspace_root).join(PACKAGE_DIR),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    }
}
