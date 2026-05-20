/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Tensor / Sparse Iso Showcase
//!
//! A heat-flow adjacency matrix arrives as a dense `CausalTensor<f64>`. Most
//! entries are zero (the graph is locally connected). The pipeline:
//!
//! 1. Sparsify the dense matrix to save memory for the next stage.
//! 2. Compute row sums on the sparse representation (fast over `values()`).
//! 3. Materialise the result back to dense for output to a downstream
//!    pipeline that expects dense tensors.
//!
//! The contrast is structural. Both `process_manual` and `process_isomorphism`
//! run the same pipeline and return the same dense tensor; only the
//! conversion path differs.
//!
//! - `process_manual` calls hand-rolled `manual_tensor_to_csr` and
//!   `manual_csr_to_tensor` helpers. Their bodies sit at the bottom of
//!   the file; count them.
//! - `process_isomorphism` uses `CsrMatrix::try_from(tensor)?` for the forward
//!   direction and `sparse.to_dense()` for the reverse. No helpers
//!   needed.
//!
//! ## Iso surface used
//!
//! - `impl<F> TryFrom<CausalTensor<F>> for CsrMatrix<F>` (Tier 1 forward;
//!   returns `Err(CsrFromTensorError { rank })` on rank ≠ 2).
//! - `impl<F> Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F>` (Tier 2
//!   reverse, on `CsrMatrix<F>` as `Self`).
//! - `CsrMatrix::to_dense()` inherent alias for the Tier 2 reverse.
//!
//! Both isos are behind the `tensor-iso` Cargo feature on
//! `deep_causality_sparse`, which this crate enables.

use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;

type F = f64;

fn main() {
    println!("=== Tensor / Sparse Iso Showcase ===\n");

    #[rustfmt::skip]
    let dense_data: Vec<F> = vec![
        2.0, 1.0, 0.0, 0.0, 0.0, 0.0,
        1.0, 2.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 2.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 2.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 2.0, 1.0,
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0,
    ];
    let dense = CausalTensor::new(dense_data, vec![6, 6]).unwrap();

    println!("Dense input (6x6):");
    print_dense(&dense);

    println!("\n--- BEFORE: hand-rolled conversion helpers ---");
    let (dense_back_before, row_sums_before) = process_manual(dense.clone());
    println!("  row sums: {:?}", row_sums_before);
    println!("  output shape: {:?}", dense_back_before.shape());

    println!("\n--- AFTER: iso-based conversion ---");
    let (dense_back_after, row_sums_after) = process_isomorphism(dense);
    println!("  row sums: {:?}", row_sums_after);
    println!("  output shape: {:?}", dense_back_after.shape());

    let drift: F = dense_back_before
        .as_slice()
        .iter()
        .zip(dense_back_after.as_slice().iter())
        .map(|(a, b)| (a - b).abs())
        .sum();
    println!("\nL1 drift between BEFORE and AFTER outputs: {:e}", drift);
    assert!(drift < 1e-12, "iso path diverged from manual path");
    assert_eq!(row_sums_before, row_sums_after);
    println!("Both paths produce the same result.\n");

    println!("--- LoC accounting ---");
    println!("BEFORE: process_manual       =  3 lines");
    println!("        manual_tensor_to_csr = 14 lines (helper below)");
    println!("        manual_csr_to_tensor = 11 lines (helper below)");
    println!("        ----------------------------------");
    println!("        total                = 28 lines");
    println!();
    println!("AFTER:  process_isomorphism  =  3 lines");
    println!("        ----------------------------------");
    println!("        total                =  3 lines");
}

// =============================================================================
// AFTER: 3 LoC of pipeline body.
// =============================================================================

fn process_isomorphism(dense: CausalTensor<F>) -> (CausalTensor<F>, Vec<F>) {
    let sparse: CsrMatrix<F> = CsrMatrix::try_from(dense).unwrap();
    let row_sums = row_sums(&sparse);
    (sparse.to_dense(), row_sums)
}

// =============================================================================
// BEFORE: 3 LoC of pipeline body + ~25 LoC of manual conversion helpers
//         (see `manual_tensor_to_csr` / `manual_csr_to_tensor` below).
// =============================================================================

fn process_manual(dense: CausalTensor<F>) -> (CausalTensor<F>, Vec<F>) {
    let sparse = manual_tensor_to_csr(&dense);
    let row_sums = row_sums(&sparse);
    (manual_csr_to_tensor(&sparse), row_sums)
}

// =============================================================================
// `row_sums` is shared between BEFORE and AFTER.
// =============================================================================

fn row_sums(sparse: &CsrMatrix<F>) -> Vec<F> {
    let (rows, _) = sparse.shape();
    let row_ptr = sparse.row_indices();
    let vals = sparse.values();
    (0..rows)
        .map(|r| vals[row_ptr[r]..row_ptr[r + 1]].iter().sum())
        .collect()
}

// =============================================================================
// BEFORE-only helpers: hand-rolled conversions using whatever CsrMatrix
// exposes publicly without the iso.
// =============================================================================

fn manual_tensor_to_csr(tensor: &CausalTensor<F>) -> CsrMatrix<F> {
    let shape = tensor.shape();
    assert_eq!(shape.len(), 2, "expected rank 2, got {}", shape.len());
    let rows = shape[0];
    let cols = shape[1];
    let data = tensor.as_slice();
    let mut triplets: Vec<(usize, usize, F)> = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let v = data[r * cols + c];
            if v != 0.0 {
                triplets.push((r, c, v));
            }
        }
    }
    CsrMatrix::from_triplets(rows, cols, &triplets).unwrap()
}

fn manual_csr_to_tensor(sparse: &CsrMatrix<F>) -> CausalTensor<F> {
    let (rows, cols) = sparse.shape();
    let row_ptr = sparse.row_indices();
    let col_idx = sparse.col_indices();
    let vals = sparse.values();
    let mut data = vec![0.0_f64; rows * cols];
    for r in 0..rows {
        for k in row_ptr[r]..row_ptr[r + 1] {
            data[r * cols + col_idx[k]] = vals[k];
        }
    }
    CausalTensor::new(data, vec![rows, cols]).unwrap()
}

// =============================================================================
// Pretty-printing helper used by `main`.
// =============================================================================

fn print_dense(t: &CausalTensor<F>) {
    let shape = t.shape();
    let rows = shape[0];
    let cols = shape[1];
    let data = t.as_slice();
    for r in 0..rows {
        let row: Vec<String> = (0..cols)
            .map(|c| format!("{:>5.1}", data[r * cols + c]))
            .collect();
        println!("  [{}]", row.join(" "));
    }
}
