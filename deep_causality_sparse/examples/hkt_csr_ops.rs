/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, Monad};
use deep_causality_sparse::{CsrMatrix, CsrMatrixWitness};

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
    // 3. Monad: Chaining operations
    // ------------------------------------------------------------------------
    println!("\n--- Monad (bind) ---");

    // Bind allows us to map a value to a new Matrix and flatten the result.
    // Example: Expand each non-zero element `x` into two elements `x` and `x+0.1` in a larger row.
    let expanded = CsrMatrixWitness::bind(pure_mat, |x| {
        // Return a 1x2 matrix for each element
        let t = vec![(0, 0, x), (0, 1, x + 0.1)];
        CsrMatrix::from_triplets(1, 2, &t).unwrap()
    });

    println!("Expanded Matrix (via bind):");
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

fn print_matrix<T: std::fmt::Display + Copy + std::fmt::Debug>(matrix: &CsrMatrix<T>) {
    let (rows, cols) = matrix.shape();
    println!("  Shape: ({}, {})", rows, cols);
    println!("  Values: {:?}", matrix.values());
    println!("  Col Indices: {:?}", matrix.col_indices());
    println!("  Row Indices: {:?}", matrix.row_indices());
}
