/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, Pure};
use deep_causality_linear::{CsrMatrix, CsrMatrixWitness};

fn main() {
    println!("=== Higher-Kinded Type (HKT) Operations on CsrMatrix ===");

    // ------------------------------------------------------------------------
    // 1. Functor: Mapping over values
    // ------------------------------------------------------------------------
    println!("\n--- Functor (fmap) ---");
    let triplets = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).expect("Failed to create matrix");

    println!("Original Matrix:");
    print_matrix(&matrix);

    // Use fmap to double every value in the sparse matrix
    let doubled = CsrMatrixWitness::fmap(matrix, |x| x * 2.0);

    println!("Doubled Matrix (via fmap):");
    print_matrix(&doubled);

    // ------------------------------------------------------------------------
    // 2. Applicative: Wrapping values
    // ------------------------------------------------------------------------
    println!("\n--- Applicative (pure & apply) ---");

    // Pure: Lift a value into a CsrMatrix context (creates a 1x1 matrix)
    let pure_mat = CsrMatrixWitness::pure(42.0);
    println!("Pure(42.0):");
    print_matrix(&pure_mat);

    // Apply: Broadcast a function over a matrix
    // First, verify we can lift a function (closure)
    // Note: Anonymous closures do not satisfy FieldConstraint unless coerced to fn ptr or Box<dyn Fn>
    let func_ptr: fn(f64) -> f64 = |x: f64| x + 10.0;
    let func_mat = CsrMatrixWitness::pure(func_ptr);

    // Apply the function matrix to the original matrix
    // Note: Our implementation of 'apply' for CsrMatrix broadcasts the function
    // if the function matrix is 1x1 (singleton).
    let added_ten = CsrMatrixWitness::apply(func_mat, doubled.clone());

    println!("Doubled Matrix + 10.0 (via apply/broadcast):");
    print_matrix(&added_ten);

    // ------------------------------------------------------------------------
    // 3. Expanding each entry -- and why this is not a Monad
    // ------------------------------------------------------------------------
    println!("\n--- Expanding each entry (no Monad) ---");

    // A shaped container cannot be a lawful Monad. `pure` has to pick a shape for a single value,
    // and the only defensible choice is the 1x1; right identity `bind(m, pure) == m` then requires
    // `bind` to reassemble an m x n matrix out of m*n one-by-ones, which a `bind` general enough to
    // accept any `f` cannot do. Measured on the implementation that used to be here: a 1x3 row with
    // a gap came back with its entry moved from column 2 to column 1, silently.
    //
    // The operation itself is fine -- it is a flat-map into a new row, not a monadic bind -- so it
    // is written directly. `openspec/notes/unified_math/HKT-LAW-FINDINGS.md` carries the measurement.
    let expanded = expand_each(&pure_mat, |x| [x, x + 0.1]);

    println!("Expanded Matrix (each entry x -> x, x + 0.1):");
    print_matrix(&expanded);

    // ------------------------------------------------------------------------
    // 4. CoMonad: Contextual Computation
    // ------------------------------------------------------------------------
    println!("\n--- CoMonad (extract & extend) ---");

    // Extract: Get the value from the current "focus" (0,0 in our simplification)
    let val = CsrMatrixWitness::extract(&expanded);
    println!("Extracted value (from 0,0): {}", val);

    // Extend: Compute over the context.
    let summed_context = CsrMatrixWitness::extend(&expanded, |m: &CsrMatrix<f64>| {
        CsrMatrixWitness::fold(m.clone(), 0.0, |acc, x| acc + x)
    });

    println!("Contextual Sum (via extend):");
    print_matrix(&summed_context);

    // ------------------------------------------------------------------------
    // 5. Foldable: Aggregation
    // ------------------------------------------------------------------------
    println!("\n--- Foldable (fold) ---");
    let total_sum = CsrMatrixWitness::fold(expanded, 0.0, |acc, x| acc + x);
    println!("Total Sum of Expanded Matrix: {}", total_sum);
}

/// Expands every stored entry into several, laid out as one row.
///
/// The flat-map a shaped container can offer honestly: the caller names the output shape by how
/// many values each entry becomes, rather than a `bind` inferring one and discarding the input's.
fn expand_each<const N: usize>(m: &CsrMatrix<f64>, f: impl Fn(f64) -> [f64; N]) -> CsrMatrix<f64> {
    let mut triplets = Vec::new();
    let mut col = 0usize;
    for &v in m.values() {
        for out in f(v) {
            triplets.push((0, col, out));
            col += 1;
        }
    }
    CsrMatrix::from_triplets(1, col, &triplets).expect("built from its own column count")
}

fn print_matrix<T: std::fmt::Display + Copy + std::fmt::Debug>(matrix: &CsrMatrix<T>) {
    let (rows, cols) = matrix.shape();
    println!("  Shape: ({}, {})", rows, cols);
    println!("  Values: {:?}", matrix.values());
    println!("  Col Indices: {:?}", matrix.col_indices());
    println!("  Row Indices: {:?}", matrix.row_indices());
}
