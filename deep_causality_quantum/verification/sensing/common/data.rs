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

/// The rows of a CSV file below its header line, each a number per header column.
///
/// # Errors
///
/// The file's, and [`std::io::ErrorKind::InvalidData`] for a file without a header line or
/// without a row below it, and naming the line of a row whose cell count differs from the
/// header's or whose cell does not parse as a number.
pub fn load_csv(path: &Path) -> std::io::Result<Vec<Vec<f64>>> {
    let text = std::fs::read_to_string(path)?;
    let invalid = |line: usize, what: String| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{}, line {line}: {what}", path.display()),
        )
    };
    let mut lines = text.lines();
    let columns = lines
        .next()
        .ok_or_else(|| invalid(1, "no header".into()))?
        .split(',')
        .count();
    let rows = lines
        .enumerate()
        .map(|(i, line)| {
            let row = line
                .split(',')
                .map(|cell| cell.trim().parse::<f64>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| invalid(i + 2, format!("{e} in '{line}'")))?;
            if row.len() == columns {
                Ok(row)
            } else {
                Err(invalid(
                    i + 2,
                    format!("{} cells against the header's {columns}", row.len()),
                ))
            }
        })
        .collect::<std::io::Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Err(invalid(2, "no rows below the header".into()));
    }
    Ok(rows)
}
