/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Source-scanning audit: no Manifold code reaches into a concrete complex's
//! private operator caches. All boundary / coboundary reads MUST go through the
//! `ChainComplex` trait (via `Cow::Borrowed` on `SimplicialComplex`).
//!
//! Mirror of `tests/traits/chain_complex_static_dispatch_tests.rs` from Stage A.

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = workspace_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

fn assert_no_direct_op_access(rel: &str) {
    let body = read(rel);
    for (lineno, line) in body.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }
        for needle in &["complex.coboundary_operators", "complex.boundary_operators"] {
            assert!(
                !line.contains(needle),
                "{rel}:{}: direct field access `{needle}` found — must go through ChainComplex trait: {line}",
                lineno + 1
            );
        }
    }
}

#[test]
fn no_direct_op_access_in_exterior() {
    assert_no_direct_op_access("src/types/manifold/differential/exterior.rs");
}

#[test]
fn no_direct_op_access_in_codifferential() {
    assert_no_direct_op_access("src/types/manifold/differential/codifferential.rs");
}

#[test]
fn no_direct_op_access_in_hodge() {
    assert_no_direct_op_access("src/types/manifold/differential/hodge.rs");
}

#[test]
fn no_direct_op_access_in_laplacian() {
    assert_no_direct_op_access("src/types/manifold/differential/laplacian.rs");
}

#[test]
fn no_direct_op_access_in_utils_manifold() {
    assert_no_direct_op_access("src/types/manifold/utils/utils_manifold.rs");
}
