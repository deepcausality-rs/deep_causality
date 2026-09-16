/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Resolves this example's own directory under both build systems.

use std::path::PathBuf;

/// This example's directory, relative to the workspace root.
const EXAMPLE_DIR: &str = "examples/physics_examples/chronometric_gm_recovery";

/// This example's directory name inside the `physics_examples` package.
const EXAMPLE_NAME: &str = "chronometric_gm_recovery";

/// The directory holding this example's sources and bundled GNSS fixtures, resolved at run time.
///
/// Every lookup is a run-time one. `env!("CARGO_MANIFEST_DIR")` would bake the compile-time
/// path into the binary, and under Bazel that path names a rustc sandbox that is gone by the
/// time the binary runs; rules_rs rejects the resulting artifact. Cargo exports
/// `CARGO_MANIFEST_DIR` into the process it launches for `cargo run`, and Bazel exports the
/// workspace root as `BUILD_WORKSPACE_DIRECTORY`, so neither needs anything embedded. Cargo's
/// variable names the `physics_examples` package directory, one level above this example, so
/// the two build systems append different suffixes to land on the same directory in the source
/// tree and read the bundled GNSS fixtures from the same place either way.
pub fn example_dir() -> PathBuf {
    if let Some(workspace_root) = std::env::var_os("BUILD_WORKSPACE_DIRECTORY") {
        return PathBuf::from(workspace_root).join(EXAMPLE_DIR);
    }

    if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        return PathBuf::from(manifest_dir).join(EXAMPLE_NAME);
    }

    // Neither build system is driving: resolve against the current directory, which is the
    // workspace root for a binary invoked from there.
    PathBuf::from(EXAMPLE_DIR)
}
