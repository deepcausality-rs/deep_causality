/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The HKT laws, per witness.
//!
//! An impl that does not satisfy its laws is worse than no impl: it composes, and produces wrong
//! answers only when a caller relies on the law.

use deep_causality_haft::{CoMonad, Functor, Monad, Pure};
use deep_causality_linear::{
    DenseMatrix, DenseMatrixWitness, DenseVector, DenseVectorWitness, MatrixView,
};

fn matrix() -> DenseMatrix<i64> {
    DenseMatrix::from_vec(vec![1i64, 2, 3, 4], 2, 2).unwrap()
}

fn vector() -> DenseVector<i64> {
    DenseVector::from_vec(vec![1i64, 2, 3])
}

// ---- functor -----------------------------------------------------------------------------------

#[test]
fn test_matrix_functor_identity() {
    let m = matrix();
    let mapped = DenseMatrixWitness::fmap(m.clone(), |x| x);
    assert_eq!(mapped, m);
}

#[test]
fn test_vector_functor_identity() {
    let v = vector();
    let mapped = DenseVectorWitness::fmap(v.clone(), |x| x);
    assert_eq!(mapped, v);
}

#[test]
fn test_matrix_functor_composition() {
    let f = |x: i64| x + 1;
    let g = |x: i64| x * 2;
    let composed = DenseMatrixWitness::fmap(matrix(), move |x| g(f(x)));
    let sequential = DenseMatrixWitness::fmap(DenseMatrixWitness::fmap(matrix(), f), g);
    assert_eq!(composed, sequential);
}

#[test]
fn test_vector_functor_composition() {
    let f = |x: i64| x + 1;
    let g = |x: i64| x * 2;
    let composed = DenseVectorWitness::fmap(vector(), move |x| g(f(x)));
    let sequential = DenseVectorWitness::fmap(DenseVectorWitness::fmap(vector(), f), g);
    assert_eq!(composed, sequential);
}

// ---- shape is preserved ------------------------------------------------------------------------

#[test]
fn test_fmap_preserves_the_matrix_dimensions() {
    let m = DenseMatrix::from_vec(vec![1i64; 6], 2, 3).unwrap();
    let mapped = DenseMatrixWitness::fmap(m, |x| x as f64);
    assert_eq!(mapped.shape(), (2, 3));
}

#[test]
fn test_fmap_preserves_the_vector_length() {
    let mapped = DenseVectorWitness::fmap(vector(), |x| x as f64);
    assert_eq!(mapped.len(), 3);
}

// ---- monad -------------------------------------------------------------------------------------

#[test]
fn test_matrix_monad_left_identity() {
    // bind(pure(a), f) == f(a)
    let f = |x: i64| DenseMatrix::from_vec(vec![x * 2], 1, 1).unwrap();
    let lhs = DenseMatrixWitness::bind(DenseMatrixWitness::pure(5i64), f);
    assert_eq!(lhs, f(5));
}

#[test]
fn test_matrix_monad_right_identity() {
    // bind(m, pure) == m
    let m = matrix();
    let bound = DenseMatrixWitness::bind(m.clone(), DenseMatrixWitness::pure);
    assert_eq!(bound, m);
}

#[test]
fn test_vector_monad_left_identity() {
    let f = |x: i64| DenseVector::from_vec(vec![x * 2]);
    let lhs = DenseVectorWitness::bind(DenseVectorWitness::pure(5i64), f);
    assert_eq!(lhs, f(5));
}

#[test]
fn test_vector_monad_right_identity() {
    let v = vector();
    assert_eq!(
        DenseVectorWitness::bind(v.clone(), DenseVectorWitness::pure),
        v
    );
}

#[test]
fn test_vector_monad_associativity() {
    // bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))
    let f = |x: i64| DenseVector::from_vec(vec![x + 1]);
    let g = |x: i64| DenseVector::from_vec(vec![x * 3]);
    let left = DenseVectorWitness::bind(DenseVectorWitness::bind(vector(), f), g);
    let right = DenseVectorWitness::bind(vector(), move |x| DenseVectorWitness::bind(f(x), g));
    assert_eq!(left, right);
}

// ---- comonad -----------------------------------------------------------------------------------

#[test]
fn test_matrix_comonad_extend_extract_is_the_identity() {
    let m = matrix();
    let extended = DenseMatrixWitness::extend(&m, DenseMatrixWitness::extract);
    assert_eq!(extended, m);
}

#[test]
fn test_vector_comonad_extend_extract_is_the_identity() {
    let v = vector();
    let extended = DenseVectorWitness::extend(&v, DenseVectorWitness::extract);
    assert_eq!(extended, v);
}

#[test]
fn test_extract_reads_the_first_entry() {
    assert_eq!(DenseMatrixWitness::extract(&matrix()), 1);
    assert_eq!(DenseVectorWitness::extract(&vector()), 1);
}

#[test]
fn test_pure_then_extract_round_trips() {
    let m: DenseMatrix<i64> = DenseMatrixWitness::pure(42);
    assert_eq!(DenseMatrixWitness::extract(&m), 42);
    let v: DenseVector<i64> = DenseVectorWitness::pure(42);
    assert_eq!(DenseVectorWitness::extract(&v), 42);
}
