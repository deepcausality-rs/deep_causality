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
// recorded in openspec/notes/archive/unified_math/HKT-LAW-FINDINGS.md rather than reproduced here.
//
// A vector has no shape beyond its length, so its `bind` is list concatenation and the laws hold.

#[test]
fn test_vector_monad_left_identity() {
    // `f` must return more than one element. With a singleton `f`, `bind` is indistinguishable
    // from `fmap` and a `bind` that kept only the head of each `f(x)` would pass.
    let f = |x: i64| DenseVector::from_vec(vec![x, x + 1]);
    let lhs = DenseVectorWitness::bind(DenseVectorWitness::pure(5i64), f);
    assert_eq!(lhs, f(5));
    assert_eq!(lhs.as_slice(), &[5, 6]);
}

#[test]
fn test_vector_monad_right_identity() {
    for v in [
        vector(),
        DenseVector::from_vec(vec![]),
        DenseVector::from_vec(vec![7i64]),
        DenseVector::from_vec(vec![3i64, 3, 3, 3, 3]),
        DenseVector::from_vec(vec![i64::MIN, 0, i64::MAX]),
    ] {
        assert_eq!(
            DenseVectorWitness::bind(v.clone(), DenseVectorWitness::pure),
            v,
            "right identity for {:?}",
            v.as_slice()
        );
    }
}

#[test]
fn test_vector_monad_associativity() {
    // bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))
    //
    // Both `f` and `g` return two elements, so the concatenation the list monad is about is
    // exercised on each side. With singleton returns the law degenerates to function
    // composition and a `bind` that dropped all but the first element still satisfies it.
    let f = |x: i64| DenseVector::from_vec(vec![x, x + 1]);
    let g = |x: i64| DenseVector::from_vec(vec![x * 3, x * 3 + 1]);
    let left = DenseVectorWitness::bind(DenseVectorWitness::bind(vector(), f), g);
    let right = DenseVectorWitness::bind(vector(), move |x| DenseVectorWitness::bind(f(x), g));
    assert_eq!(left, right);
    assert_eq!(left.len(), 12, "3 elements, each expanding by 2 then by 2");
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
    // `extract` reads the first element, so the round trip alone holds for any `pure` whose
    // first element is the value: a `pure` returning [42, 0, 0] would pass. The shape `pure`
    // chooses is the decision worth pinning, so pin it.
    let m: DenseMatrix<i64> = DenseMatrixWitness::pure(42);
    assert_eq!(m.shape(), (1, 1), "pure builds the 1x1");
    assert_eq!(DenseMatrixWitness::extract(&m), 42);
    let v: DenseVector<i64> = DenseVectorWitness::pure(42);
    assert_eq!(v.len(), 1, "pure builds the one-element vector");
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
    // Distinct per-position functions. With four copies of one function any permutation of the
    // pairing yields the same answer, so the assertion could not see a mispairing. The shape
    // assertion that used to sit here could not fail either: `apply` reads the shape off `fa`
    // and rebuilds with it, so whenever it returns at all the shape is `fa`'s.
    let fns: DenseMatrix<fn(i64) -> i64> = DenseMatrix::from_vec(
        vec![
            (|x| x + 10) as fn(i64) -> i64,
            |x| x + 20,
            |x| x + 30,
            |x| x + 40,
        ],
        2,
        2,
    )
    .unwrap();
    let applied = DenseMatrixWitness::apply(fns, matrix());
    assert_eq!(applied.as_slice(), &[11, 22, 33, 44]);
}

#[test]
fn test_matrix_applicative_identity() {
    use deep_causality_haft::Applicative;
    // apply(pure(id), v) == v. This is the law that actually pins shape behaviour.
    let idf: DenseMatrix<fn(i64) -> i64> = DenseMatrixWitness::pure((|x| x) as fn(i64) -> i64);
    assert_eq!(DenseMatrixWitness::apply(idf, matrix()), matrix());
}

#[test]
fn test_vector_applicative_is_cartesian() {
    use deep_causality_haft::Applicative;
    // `DenseVectorWitness` carries `Monad`, so coherence with its list-concatenation `bind` pins
    // `apply` to the cartesian product, function-major.
    let fns: DenseVector<fn(i64) -> i64> =
        DenseVector::from_vec(vec![(|x| x * 10) as fn(i64) -> i64, |x| x * 100, |x| {
            x * 1000
        }]);
    let applied = DenseVectorWitness::apply(fns, vector());
    assert_eq!(
        applied.as_slice(),
        &[10, 20, 30, 100, 200, 300, 1000, 2000, 3000]
    );
}

#[test]
fn test_zip_vector_applies_pointwise() {
    use deep_causality_haft::MonoidalApplicative;
    use deep_causality_linear::ZipDenseVectorWitness;
    // The elementwise reading `DenseVectorWitness` cannot give without breaking coherence.
    let fns: DenseVector<fn(i64) -> i64> =
        DenseVector::from_vec(vec![(|x| x * 10) as fn(i64) -> i64, |x| x * 100, |x| {
            x * 1000
        }]);
    let applied = ZipDenseVectorWitness::apply(fns, vector());
    assert_eq!(applied.as_slice(), &[10, 200, 3000]);
}

#[test]
fn test_zip_vector_semigroupal_associativity() {
    use deep_causality_haft::{Functor, Semigroupal};
    use deep_causality_linear::ZipDenseVectorWitness;
    let a = DenseVector::from_vec(vec![1i64, 2]);
    let b = DenseVector::from_vec(vec![3i64, 4]);
    let c = DenseVector::from_vec(vec![5i64, 6]);
    let left =
        ZipDenseVectorWitness::zip(ZipDenseVectorWitness::zip(a.clone(), b.clone()), c.clone());
    let right = ZipDenseVectorWitness::zip(a, ZipDenseVectorWitness::zip(b, c));
    let reassociated = ZipDenseVectorWitness::fmap(left, |((x, y), z)| (x, (y, z)));
    assert_eq!(reassociated, right);
}

#[test]
fn test_vector_applicative_identity() {
    use deep_causality_haft::Applicative;
    // apply(pure(id), v) == v.
    let idf: DenseVector<fn(i64) -> i64> = DenseVectorWitness::pure((|x| x) as fn(i64) -> i64);
    assert_eq!(DenseVectorWitness::apply(idf, vector()), vector());
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

#[test]
fn test_vector_applicative_monad_coherence() {
    use deep_causality_haft::Applicative;
    // `DenseVectorWitness` carries both `Monad` and `Applicative`, so it owes
    // apply(ff, fa) == bind(ff, |f| fmap(fa, f)).
    // `bind` is list concatenation, which runs the continuation once per function, so coherence
    // admits only the cartesian applicative.
    let fns: DenseVector<fn(i64) -> i64> =
        DenseVector::from_vec(vec![(|x| x * 10) as fn(i64) -> i64, |x| x + 100]);
    let fa = vector();
    let fa2 = fa.clone();

    let via_apply = DenseVectorWitness::apply(fns.clone(), fa);
    let via_bind = DenseVectorWitness::bind(fns, move |f: fn(i64) -> i64| {
        DenseVectorWitness::fmap(fa2.clone(), f)
    });
    assert_eq!(via_apply, via_bind);
}
