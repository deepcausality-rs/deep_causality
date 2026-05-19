/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Static-dispatch audit for the `ChainComplex` trait.
//!
//! Asserts that no `dyn Iterator` survives in the trait surface or its impl files.
//! This guards against re-introducing dynamic dispatch on the cell-iteration path
//! (AGENTS.md §"Static Dispatch").

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points at deep_causality_topology/ during cargo test.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = workspace_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

fn assert_no_dyn_iterator(rel: &str) {
    let body = read(rel);
    for (lineno, line) in body.lines().enumerate() {
        // Allow the substring to appear inside comments that explicitly mention the
        // historical pattern; flag any code occurrence.
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }
        assert!(
            !line.contains("dyn Iterator"),
            "{rel}:{}: `dyn Iterator` found in code: {line}",
            lineno + 1
        );
    }
}

#[test]
fn no_dyn_iterator_in_chain_complex_trait() {
    assert_no_dyn_iterator("src/traits/chain_complex.rs");
}

#[test]
fn no_dyn_iterator_in_lattice_impl() {
    assert_no_dyn_iterator("src/types/lattice/mod.rs");
}

#[test]
fn no_dyn_iterator_in_cell_complex_impl() {
    assert_no_dyn_iterator("src/types/cell_complex/mod.rs");
}

#[test]
fn no_dyn_iterator_in_simplicial_chain_complex_impl() {
    assert_no_dyn_iterator("src/types/simplicial_complex/topology/chain_complex_impl.rs");
}
