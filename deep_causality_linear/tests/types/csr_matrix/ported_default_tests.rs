/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Default` for `CsrMatrix`, ported from `deep_causality_sparse`.
//!
//! The claim under test is that `default()` and `new()` build the same empty matrix, and that the
//! empty matrix they build satisfies the CSR invariants.

use deep_causality_linear::CsrMatrix;

#[test]
fn test_default_is_new() {
    let default_matrix: CsrMatrix<f64> = CsrMatrix::default();
    let new_matrix: CsrMatrix<f64> = CsrMatrix::new();
    assert_eq!(default_matrix.shape(), new_matrix.shape());
    assert!(default_matrix.values().is_empty());
    assert!(new_matrix.values().is_empty());
}

#[test]
fn test_default_agrees_with_new_on_every_field() {
    let default_matrix: CsrMatrix<f64> = CsrMatrix::default();
    let new_matrix: CsrMatrix<f64> = CsrMatrix::new();
    assert_eq!(default_matrix, new_matrix);
}

#[test]
fn test_default_is_the_empty_shape() {
    let m: CsrMatrix<f64> = CsrMatrix::default();
    assert_eq!(m.shape(), (0, 0));
    assert!(m.values().is_empty());
    assert!(m.col_indices().is_empty());
}

#[test]
fn test_default_carries_no_row_pointers() {
    // The `rows + 1` invariant governs a matrix with rows. The default has none, and the crate this
    // moves from returns an empty pointer array here; `row_indices()` is public, so that is a
    // result a caller can see and not an implementation detail.
    let m: CsrMatrix<f64> = CsrMatrix::default();
    assert!(m.row_indices().is_empty());
    assert!(m.col_indices().is_empty());
    assert!(m.values().is_empty());
}

#[test]
fn test_default_and_with_capacity_disagree_on_the_empty_row_pointer() {
    // An inconsistency inherited from the crate this moves from, reproduced deliberately.
    //
    // `new()` and `default()` leave the row-pointer array empty; `with_capacity(0, 0, 0)` builds
    // `vec![0; rows + 1]`, which is `[0]`. Two zero-shaped matrices that compare unequal.
    //
    //   sparse new().row_indices()              = []
    //   sparse with_capacity(0,0,0).row_indices = [0]
    //   sparse new() == with_capacity(0,0,0)    : false
    //
    // Probed against the published crate rather than read off its source. Reproducing it keeps the
    // move faithful; `linear-matrix-representations` requires that code written against the old
    // type get identical results, and a caller comparing two empty matrices gets `false` today.
    //
    // Recorded in openspec/notes/archive/linear/PORTING-FINDINGS.md as a candidate to settle in phase 5,
    // where the old surface is retired and changing it costs nothing.
    let default_made: CsrMatrix<f64> = CsrMatrix::default();
    let capacity_made: CsrMatrix<f64> = CsrMatrix::with_capacity(0, 0, 0);

    assert!(default_made.row_indices().is_empty());
    assert_eq!(capacity_made.row_indices(), &vec![0]);
    assert_ne!(
        default_made, capacity_made,
        "the two disagree, as they do in the crate this moves from"
    );

    // What they do agree on, which is everything a caller can actually use.
    assert_eq!(default_made.shape(), capacity_made.shape());
    assert!(default_made.values().is_empty() && capacity_made.values().is_empty());
    assert!(default_made.col_indices().is_empty() && capacity_made.col_indices().is_empty());
}
