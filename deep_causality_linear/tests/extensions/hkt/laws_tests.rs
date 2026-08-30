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
//
// Only the vector claims `Monad`, so only the vector is tested for its laws.
//
// `DenseMatrixWitness` does not claim it. `pure` must choose a shape for one value and a shaped
// container has no canonical one; taking the 1x1, right identity would need `bind` to reassemble an
// `m x n` out of `m*n` one-by-ones, which no general `bind` can do. `deep_causality_sparse`'s
// witness claims `Monad` and fails this law — `bind(m, pure)` turns a 2x2 into a 1x4 — which is
// recorded in openspec/notes/unified_math/HKT-LAW-FINDINGS.md rather than reproduced here.
//
// A vector has no shape beyond its length, so its `bind` is list concatenation and the laws hold.

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

// ---- foldable and applicative ------------------------------------------------------------------

#[test]
fn test_matrix_fold_visits_every_entry() {
    use deep_causality_haft::Foldable;
    let total = DenseMatrixWitness::fold(matrix(), 0i64, |acc, v| acc + v);
    assert_eq!(total, 1 + 2 + 3 + 4);
}

#[test]
fn test_vector_fold_visits_every_entry() {
    use deep_causality_haft::Foldable;
    let total = DenseVectorWitness::fold(vector(), 0i64, |acc, v| acc + v);
    assert_eq!(total, 1 + 2 + 3);
}

#[test]
fn test_fold_respects_the_initial_value() {
    use deep_causality_haft::Foldable;
    assert_eq!(
        DenseVectorWitness::fold(vector(), 100i64, |acc, v| acc + v),
        106
    );
}

#[test]
fn test_fold_over_an_empty_container_returns_the_initial_value() {
    use deep_causality_haft::Foldable;
    let empty: DenseVector<i64> = DenseVector::from_vec(vec![]);
    assert_eq!(DenseVectorWitness::fold(empty, 42i64, |acc, v| acc + v), 42);
}

#[test]
fn test_matrix_applicative_applies_pointwise() {
    use deep_causality_haft::Applicative;
    let fns: DenseMatrix<fn(i64) -> i64> =
        DenseMatrix::from_vec(vec![(|x| x + 1) as fn(i64) -> i64; 4], 2, 2).unwrap();
    let applied = DenseMatrixWitness::apply(fns, matrix());
    assert_eq!(applied.as_slice(), &[2, 3, 4, 5]);
    assert_eq!(applied.shape(), (2, 2), "apply preserves the shape");
}

#[test]
fn test_vector_applicative_applies_pointwise() {
    use deep_causality_haft::Applicative;
    let fns: DenseVector<fn(i64) -> i64> =
        DenseVector::from_vec(vec![(|x| x * 10) as fn(i64) -> i64; 3]);
    let applied = DenseVectorWitness::apply(fns, vector());
    assert_eq!(applied.as_slice(), &[10, 20, 30]);
}

#[test]
fn test_vector_bind_concatenates() {
    use deep_causality_haft::Monad;
    // The list monad: each element expands, and the pieces join.
    let doubled = DenseVectorWitness::bind(vector(), |x| DenseVector::from_vec(vec![x, x]));
    assert_eq!(doubled.as_slice(), &[1, 1, 2, 2, 3, 3]);
}

#[test]
fn test_extend_sees_each_position_in_turn() {
    use deep_causality_haft::CoMonad;
    // The shifted view is what makes extend(extract) the identity; this checks the focus moves.
    let firsts = DenseVectorWitness::extend(&vector(), |v| v.as_slice()[0]);
    assert_eq!(firsts.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_matrix_extend_sees_each_position_in_turn() {
    use deep_causality_haft::CoMonad;
    let firsts = DenseMatrixWitness::extend(&matrix(), |m| m.as_slice()[0]);
    assert_eq!(firsts.as_slice(), &[1, 2, 3, 4]);
}
