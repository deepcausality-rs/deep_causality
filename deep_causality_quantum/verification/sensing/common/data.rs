/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reading digitised data that ships with the crate.

use std::path::{Path, PathBuf};

/// This crate's directory, relative to the workspace root.
const PACKAGE_DIR: &str = "deep_causality_quantum";

/// The directory holding this crate's `Cargo.toml`, resolved at run time: under the workspace root
/// that `bazel run` sets in `BUILD_WORKSPACE_DIRECTORY`, at `CARGO_MANIFEST_DIR` under `cargo run`,
/// and under the current directory otherwise. A compile-time `env!("CARGO_MANIFEST_DIR")` would name
/// a rustc sandbox under Bazel.
pub fn manifest_dir() -> PathBuf {
    if let Some(workspace_root) = std::env::var_os("BUILD_WORKSPACE_DIRECTORY") {
        return PathBuf::from(workspace_root).join(PACKAGE_DIR);
    }
    if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        return PathBuf::from(manifest_dir);
    }
    PathBuf::from(PACKAGE_DIR)
}

/// The numeric rows of a CSV file; a row that does not parse, such as a header, is skipped.
///
/// # Errors
///
/// The file's.
pub fn load_csv(path: &Path) -> std::io::Result<Vec<Vec<f64>>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .filter_map(|line| {
            line.split(',')
                .map(|cell| cell.trim().parse::<f64>())
                .collect::<Result<Vec<_>, _>>()
                .ok()
        })
        .filter(|row| !row.is_empty())
        .collect())
}
