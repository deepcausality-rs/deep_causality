/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Resolves this example package's root directory under both build systems.

use std::path::PathBuf;

/// This package's directory, relative to the workspace root.
const PACKAGE_DIR: &str = "examples/causal_discovery_examples";

/// The directory holding this package's `Cargo.toml`, resolved at run time.
///
/// Every lookup here is a run-time one. `env!("CARGO_MANIFEST_DIR")` would bake the
/// compile-time path into the binary, and under Bazel that path names a rustc sandbox that
/// no longer exists when the binary runs; rules_rs rejects the resulting artifact outright.
/// Cargo exports `CARGO_MANIFEST_DIR` into the process it launches for `cargo run`, and Bazel
/// exports the workspace root as `BUILD_WORKSPACE_DIRECTORY`, so both build systems can be
/// served without embedding anything. Both land on the same directory in the source tree, so
/// the examples read the bundled Sock Shop cases from the same place either way.
pub fn manifest_dir() -> PathBuf {
    if let Some(workspace_root) = std::env::var_os("BUILD_WORKSPACE_DIRECTORY") {
        return PathBuf::from(workspace_root).join(PACKAGE_DIR);
    }

    if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        return PathBuf::from(manifest_dir);
    }

    // Neither build system is driving: resolve against the current directory, which is the
    // workspace root for a binary invoked from there.
    PathBuf::from(PACKAGE_DIR)
}
