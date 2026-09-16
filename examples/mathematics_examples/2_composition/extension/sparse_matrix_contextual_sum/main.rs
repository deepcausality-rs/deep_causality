/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # CoMonadic extension over a sparse matrix
//!
//! `CsrMatrixWitness` carries `Functor`, `Pure`, `Applicative`, `CoMonad` and `Foldable`, so a
//! sparse matrix takes the same vocabulary as a tensor or a manifold. `extend` is the one that
//! reaches: it hands the whole matrix to a closure at every position, and the closure may call
//! into any crate it likes -- here it folds the context back down to a scalar.

use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, Pure};
use deep_causality_linear::{CsrMatrix, CsrMatrixWitness};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift, lower};
use std::fmt::Debug;

/// The working scalar. Every stored entry of the matrix carries it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE_TENTH: FloatType = const_scalar_from_float!(FloatType, 0.1);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const THREE: FloatType = const_scalar_from_int!(FloatType, 3);
const TEN: FloatType = const_scalar_from_int!(FloatType, 10);

fn main() {
    // 1. Functor: fmap walks the stored entries and leaves the sparsity pattern alone.
    let triplets = vec![(0, 0, ONE), (0, 2, TWO), (1, 1, THREE)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).expect("three triplets inside a 2x3");
    let original = matrix.clone();
    let doubled = CsrMatrixWitness::fmap(matrix, |x| x * TWO);
    print_functor(&original, &doubled);

    // 2. Pure lifts a scalar into a 1x1; apply broadcasts a one-entry function matrix.
    let pure_mat = CsrMatrixWitness::pure(lift::<FloatType>(42.0));
    // An anonymous closure needs coercing to a fn pointer before it can be a matrix payload.
    let func_ptr: fn(FloatType) -> FloatType = |x| x + TEN;
    let func_mat = CsrMatrixWitness::pure(func_ptr);
    let added_ten = CsrMatrixWitness::apply(func_mat, doubled.clone());
    print_applicative(&pure_mat, &added_ten);

    // 3. Expanding each entry, written directly rather than as a `bind`.
    //
    // A shaped container cannot be a lawful Monad. `pure` has to pick a shape for a single value,
    // and the only defensible choice is the 1x1; right identity `bind(m, pure) == m` then requires
    // `bind` to reassemble an m x n matrix out of m*n one-by-ones, which a `bind` general enough to
    // accept any `f` cannot do. Measured on the implementation that used to be here: a 1x3 row with
    // a gap came back with its entry moved from column 2 to column 1, silently.
    //
    // The operation itself is fine -- it is a flat-map into a new row, not a monadic bind -- so it
    // is written directly. `openspec/notes/archive/unified_math/HKT-LAW-FINDINGS.md` carries the
    // measurement.
    let expanded = expand_each(&pure_mat, |x| [x, x + ONE_TENTH]);
    print_expanded(&expanded);

    // 4. CoMonad: `extract` reads the focus, `extend` recomputes every position from the
    //    whole matrix. The kernel here folds the context down to one number.
    let val = CsrMatrixWitness::extract(&expanded);
    let summed_context = CsrMatrixWitness::extend(&expanded, |m: &CsrMatrix<FloatType>| {
        CsrMatrixWitness::fold(m.clone(), ZERO, |acc, x| acc + x)
    });
    print_comonad(val, &summed_context);

    // 5. Foldable: the same reduction applied once, to the whole matrix.
    let total_sum = CsrMatrixWitness::fold(expanded, ZERO, |acc, x| acc + x);
    print_fold(total_sum);
}

/// Expands every stored entry into several, laid out as one row.
///
/// The flat-map a shaped container can offer honestly: the caller names the output shape by how
/// many values each entry becomes, rather than a `bind` inferring one and discarding the input's.
fn expand_each<const N: usize>(
    m: &CsrMatrix<FloatType>,
    f: impl Fn(FloatType) -> [FloatType; N],
) -> CsrMatrix<FloatType> {
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

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_functor(original: &CsrMatrix<FloatType>, doubled: &CsrMatrix<FloatType>) {
    println!("=== Higher-Kinded Type (HKT) Operations on CsrMatrix ===");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("\n--- Functor (fmap) ---");
    println!("Original Matrix:");
    print_matrix(original);
    println!("Doubled Matrix (via fmap):");
    print_matrix(doubled);
}

fn print_applicative(pure_mat: &CsrMatrix<FloatType>, added_ten: &CsrMatrix<FloatType>) {
    println!("\n--- Applicative (pure & apply) ---");
    println!("Pure(42.0):");
    print_matrix(pure_mat);
    println!("Doubled Matrix + 10.0 (via apply/broadcast):");
    print_matrix(added_ten);
}

fn print_expanded(expanded: &CsrMatrix<FloatType>) {
    println!("\n--- Expanding each entry (no Monad) ---");
    println!("Expanded Matrix (each entry x -> x, x + 0.1):");
    print_matrix(expanded);
}

fn print_comonad(extracted: FloatType, summed: &CsrMatrix<FloatType>) {
    println!("\n--- CoMonad (extract & extend) ---");
    println!("Extracted value (from 0,0): {}", lower(extracted));
    println!("Contextual Sum (via extend):");
    print_matrix(summed);
}

fn print_fold(total: FloatType) {
    println!("\n--- Foldable (fold) ---");
    println!("Total Sum of Expanded Matrix: {}", lower(total));
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_matrix<T: Copy + Debug>(matrix: &CsrMatrix<T>) {
    let (rows, cols) = matrix.shape();
    println!("  Shape: ({rows}, {cols})");
    println!("  Values: {:?}", matrix.values());
    println!("  Col Indices: {:?}", matrix.col_indices());
    println!("  Row Indices: {:?}", matrix.row_indices());
}
