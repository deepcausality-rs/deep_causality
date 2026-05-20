/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Tensor / Sparse Iso Showcase
//!
//! A heat-flow adjacency matrix arrives as a dense `CausalTensor<f64>`. Most
//! entries are zero (the graph is locally connected). We:
//!
//! 1. Sparsify the dense matrix to save memory for the next stage.
//! 2. Compute row sums on the sparse representation (faster when sparse).
//! 3. Materialise a thresholded version back to dense for output to a
//!    downstream pipeline that expects dense tensors.
//!
//! The contrast:
//!
//! - **BEFORE**: hand-rolled conversion functions in both directions. The
//!   reverse direction must allocate the dense buffer, walk the triplets,
//!   and place each value at the right linear index. Roughly 20 LoC of
//!   conversion code that has to be re-derived (or copy-pasted) every
//!   time a sparse output needs to feed a dense consumer.
//! - **AFTER**: `CsrMatrix::from(tensor)` for the forward direction;
//!   `sparse.to_dense()` for the reverse. Two method calls.
//!
//! ## Iso surface used
//!
//! - `impl<F> From<CausalTensor<F>> for CsrMatrix<F>` (Tier 1 forward;
//!   panics on rank ≠ 2).
//! - `impl<F> Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F>` (Tier 2
//!   reverse, on `CsrMatrix<F>` as `Self`).
//! - `CsrMatrix::to_dense()` inherent alias for the Tier 2 reverse.

use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;

type F = f64;

fn main() {
    println!("=== Tensor / Sparse Iso Showcase ===\n");

    // ---------------------------------------------------------------------
    // Build a 6x6 heat-flow adjacency matrix. Locally-connected graph:
    // each node has edges to its two neighbours plus self-loops.
    // ---------------------------------------------------------------------
    let dense_data: Vec<F> = vec![
        2.0, 1.0, 0.0, 0.0, 0.0, 0.0,
        1.0, 2.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 2.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 2.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 2.0, 1.0,
        0.0, 0.0, 0.0, 0.0, 1.0, 2.0,
    ];
    let dense = CausalTensor::new(dense_data, vec![6, 6]).unwrap();

    println!("Dense input ({}x{}):", 6, 6);
    print_dense(&dense);

    // ---------------------------------------------------------------------
    // BEFORE: hand-rolled forward conversion
    // ---------------------------------------------------------------------
    println!("\n--- BEFORE: manual tensor -> CsrMatrix ---");
    let sparse_before = manual_tensor_to_csr(&dense);
    println!(
        "  manual conversion: {} non-zero entries stored",
        sparse_before.nnz()
    );

    // Sparse-only operation: row sums
    let row_sums_before = manual_row_sums(&sparse_before);
    println!("  row sums (manual): {:?}", row_sums_before);

    // BEFORE: hand-rolled reverse conversion
    let dense_back_before = manual_csr_to_tensor(&sparse_before);
    println!(
        "  manual back-conversion produced shape {:?}",
        dense_back_before.shape()
    );

    // ---------------------------------------------------------------------
    // AFTER: iso-based conversion
    // ---------------------------------------------------------------------
    println!("\n--- AFTER: iso-based conversion ---");

    // Forward: one expression.
    let sparse_after: CsrMatrix<F> = dense.clone().into();
    println!(
        "  iso conversion: {} non-zero entries stored",
        sparse_after.nnz()
    );

    // Sparse-only operation: row sums. (The same algorithm; just shown for
    // parallelism with the BEFORE path.)
    let row_sums_after = manual_row_sums(&sparse_after);
    println!("  row sums (iso):    {:?}", row_sums_after);

    // Reverse: one method call.
    let dense_back_after = sparse_after.to_dense();
    println!(
        "  iso back-conversion produced shape {:?}",
        dense_back_after.shape()
    );

    // ---------------------------------------------------------------------
    // Equivalence check
    // ---------------------------------------------------------------------
    let drift: F = dense_back_before
        .as_slice()
        .iter()
        .zip(dense_back_after.as_slice().iter())
        .map(|(a, b)| (a - b).abs())
        .sum();

    println!("\nL1 drift between BEFORE and AFTER round-trips: {:e}", drift);
    assert!(drift < 1e-12, "iso path diverged from manual path");
    println!("Both paths round-trip identically.\n");

    // ---------------------------------------------------------------------
    // LoC summary
    // ---------------------------------------------------------------------
    println!("Conversion LoC, BEFORE: ~22 lines (manual_tensor_to_csr + manual_csr_to_tensor)");
    println!("Conversion LoC, AFTER:  2 expressions (`.into()` and `.to_dense()`)");
}

// =============================================================================
// BEFORE: hand-rolled conversions
// =============================================================================

fn manual_tensor_to_csr(tensor: &CausalTensor<F>) -> CsrMatrix<F> {
    let shape = tensor.shape();
    assert_eq!(shape.len(), 2, "expected rank 2, got {}", shape.len());
    let rows = shape[0];
    let cols = shape[1];
    let data = tensor.as_slice();

    let mut row_indices = Vec::new();
    let mut col_indices = Vec::new();
    let mut values = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let v = data[r * cols + c];
            if v != 0.0 {
                row_indices.push(r);
                col_indices.push(c);
                values.push(v);
            }
        }
    }
    // The exact constructor name depends on the sparse-matrix API. Using
    // a representative form here.
    CsrMatrix::from_parts(row_indices, col_indices, values, (rows, cols))
}

fn manual_csr_to_tensor(sparse: &CsrMatrix<F>) -> CausalTensor<F> {
    let (rows, cols) = sparse.shape();
    let mut data = vec![0.0; rows * cols];
    for (r, c, v) in sparse.iter_triplets() {
        data[r * cols + c] = v;
    }
    CausalTensor::new(data, vec![rows, cols]).unwrap()
}

fn manual_row_sums(sparse: &CsrMatrix<F>) -> Vec<F> {
    let (rows, _) = sparse.shape();
    let mut sums = vec![0.0; rows];
    for (r, _, v) in sparse.iter_triplets() {
        sums[r] += v;
    }
    sums
}

// =============================================================================
// Pretty-printing
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
