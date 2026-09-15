/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `CsrMatrix`: the sparse-matrix API surface
//!
//! Construction from triplets, then the operations a sparse matrix is for: scalar scaling,
//! addition, matrix-vector and matrix-matrix products, transpose, and the error a dimension
//! mismatch returns rather than panics on.

use deep_causality_linear::CsrMatrix;
use deep_causality_num::{lift, lower};
use std::fmt::Display;

/// The working scalar. Every stored entry carries it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // 1. From triplets. Only the stored entries are listed; the rest is structurally zero.
    //    A = [[1, 0, 2],
    //         [0, 3, 0]]
    let a = CsrMatrix::from_triplets(2, 3, &triplets(&[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]))?;
    print_matrix_a(&a, a.get_value_at(0, 0), a.get_value_at(0, 1));

    // 2. Scalar multiplication scales the stored entries and leaves the pattern alone.
    let scalar = lift::<FloatType>(2.0);
    let b = a.scalar_mult(scalar);
    print_scalar_mult(scalar, &b);

    // 3. Addition unions the two sparsity patterns.
    //    C = [[0, 5, 0],      D = A + C = [[1, 5, 2],
    //         [6, 0, 0]]                   [6, 3, 0]]
    let c = CsrMatrix::from_triplets(2, 3, &triplets(&[(0, 1, 5.0), (1, 0, 6.0)]))?;
    let d = a.add_matrix(&c)?;
    print_addition(&c, &d);

    // 4. Matrix-vector product. y = Ax = [1*1 + 2*3, 3*2] = [7, 6].
    let x: Vec<FloatType> = [1.0, 2.0, 3.0].iter().map(|&v| lift(v)).collect();
    let y = a.vec_mult(&x)?;
    print_vec_mult(&x, &y);

    // 5. Matrix-matrix product. F = A*E is 2x2 = [[16, 0], [0, 15]].
    let e = CsrMatrix::from_triplets(3, 2, &triplets(&[(0, 0, 4.0), (1, 1, 5.0), (2, 0, 6.0)]))?;
    let f = a.mat_mult(&e)?;
    print_mat_mult(&e, &f);

    // 6. Transpose swaps the axes, so a 2x3 becomes a 3x2.
    print_transpose(&a.transpose());

    // 7. A dimension mismatch is a returned error, not a panic: x has 2 entries, A has 3 columns.
    let x_invalid: Vec<FloatType> = [1.0, 2.0].iter().map(|&v| lift(v)).collect();
    print_mismatch(a.vec_mult(&x_invalid).err().map(|e| e.to_string()));

    Ok(())
}

/// Lifts `(row, col, value)` literals into the working scalar.
fn triplets(entries: &[(usize, usize, f64)]) -> Vec<(usize, usize, FloatType)> {
    entries
        .iter()
        .map(|&(r, c, v)| (r, c, lift::<FloatType>(v)))
        .collect()
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("Demonstrating basic CsrMatrix operations:");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_matrix_a<M: Display>(a: &M, at_00: FloatType, at_01: FloatType) {
    println!("\n--- Matrix A (2x3) from triplets ---");
    println!("Matrix A:\n{a}");
    println!("A[0,0]: {}", lower(at_00));
    println!("A[0,1]: {}", lower(at_01)); // structurally zero, so not stored
}

fn print_scalar_mult<M: Display>(scalar: FloatType, b: &M) {
    println!("\n--- Scalar Multiplication (A * 2.0) ---");
    println!("Matrix B (A *{});\n{}", lower(scalar), b);
}

fn print_addition<M: Display>(c: &M, d: &M) {
    println!("\n--- Matrix Addition (A + C) ---");
    println!("Matrix C:\n{c}");
    println!("Matrix D (A + C):\n{d}");
}

fn print_vec_mult(x: &[FloatType], y: &[FloatType]) {
    println!("\n--- Matrix-Vector Multiplication (A * x) ---");
    println!("Vector x: {:?}", shown(x));
    println!("Result y = Ax: {:?}", shown(y));
}

fn print_mat_mult<M: Display>(e: &M, f: &M) {
    println!("\n--- Matrix Multiplication (A * E) ---");
    println!("Matrix E:\n{e}");
    println!("Matrix F (A * E):\n{f}");
}

fn print_transpose<M: Display>(a_t: &M) {
    println!("\n--- Transpose (A^T) ---");
    println!("Matrix A^T:\n{a_t}");
}

fn print_mismatch(error: Option<String>) {
    println!("\n--- Error Handling (Dimension Mismatch) ---");
    match error {
        Some(e) => println!("Caught expected error: {e}"),
        None => println!("Unexpected success with invalid vector length"),
    }
}

fn shown(xs: &[FloatType]) -> Vec<f64> {
    xs.iter().map(|&x| lower(x)).collect()
}
