/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `CsrMatrixWitness` higher-kinded surface, ported from `deep_causality_sparse`.
//!
//! `deep_causality_sparse` implements eight `deep_causality_haft` traits on `CsrMatrixWitness` —
//! `HKT`, `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad`, `CoMonad` and `Adjunction` — and
//! its `tests/extensions/ext_hkt_tests.rs` exercises fourteen behaviours across them.
//!
//! `deep_causality_linear` declares the witness with `HKT` alone. The remaining seven impls are
//! marked as still to follow in `src/extensions/hkt/csr_matrix_witness.rs`, and task 4.11 of the
//! `add-linear-algebra-crate` change is where `extensions/ext_hkt.rs` moves. All fourteen source
//! tests name a trait that does not resolve here, so they are held back rather than rewritten
//! against a substitute. What is portable today is the premise all fourteen rest on: the witness
//! exists and projects to `CsrMatrix<T>`.
//!
//! # Two of the held-back behaviours carry a decision, not a transcription
//!
//! Restoring them verbatim would enshrine a defect that is already on record:
//!
//! - `bind` flattens its result to `1 x count`, so `bind(m, pure)` turns a 2x2 into a 1x4 and monad
//!   right identity fails. `openspec/notes/archive/unified_math/HKT-LAW-FINDINGS.md` states the finding, and
//!   `DenseMatrixWitness` answers the same problem by declining `Monad` outright.
//! - `counit` and `right_adjunct` are written in terms of that `bind`, so the `Adjunction` impl
//!   turns on the same decision.
//!
//! The sparse behaviours the fourteen record, for whoever restores them: `fmap` maps the stored
//! entries and keeps the shape; `fold` folds the stored entries; `pure` builds a 1x1; `apply`
//! broadcasts a 1x1 function matrix and otherwise intersects two equal shapes; `extract` reads the
//! first stored entry and panics on the empty matrix; `extend` applies `f` to a view **cropped** to
//! each stored position in turn, keeping the input's structure; `unit` wraps a `ctx`-shaped inner
//! matrix carrying the value at (0, 0) in a 1x1 outer, degenerating to an empty inner when `ctx` has
//! a zero dimension.
//!
//! The crop in `extend` is the third decision owed. `DenseMatrixWitness` rotates instead, so a 1x2
//! `[10, 20]` extended with a sum gives `[30, 20]` under the sparse crop and `[30, 30]` under the
//! dense rotation. Both satisfy `extend(extract) == id`; they differ in what `f` sees.

use deep_causality_haft::{Functor, HKT};
use deep_causality_linear::{CsrMatrix, CsrMatrixWitness};

#[test]
fn test_the_witness_projects_to_the_sparse_matrix_it_stands_for() {
    // The projection itself is settled by the build: if `Type<f64>` were not `CsrMatrix<f64>`
    // the annotation below would not compile, so asserting on `shape()` and `values()` of a
    // value built by `from_triplets` reports nothing the compiler had not already refused.
    // What is worth running is a *use* of the projection through a trait method.
    let matrix: <CsrMatrixWitness as HKT>::Type<f64> =
        CsrMatrix::from_triplets(1, 3, &[(0, 0, 1.0), (0, 1, 2.0), (0, 2, 3.0)]).unwrap();

    let mapped = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(matrix.clone(), |x| x);
    assert_eq!(
        mapped, matrix,
        "the witness maps the container it projects to"
    );
}

// ---- restored: the traits the witness now carries -----------------------------------------------
//
// The port skipped fourteen tests because `CsrMatrixWitness` implemented `HKT` alone. Five of the
// seven missing traits are now present, so those tests come back. `Monad` and `Adjunction` stay
// absent — see the impl site and HKT-LAW-FINDINGS.md.

#[cfg(test)]
mod restored {
    use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, Pure};
    use deep_causality_linear::{CsrMatrix, CsrMatrixWitness};

    fn m() -> CsrMatrix<f64> {
        CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 2.0), (1, 1, 3.0)]).unwrap()
    }

    #[test]
    fn test_fmap_maps_the_stored_entries_and_keeps_the_shape() {
        let doubled = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(m(), |x| x * 2.0);
        assert_eq!(doubled.shape(), (2, 2));
        assert_eq!(doubled.values(), &vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_fmap_leaves_the_structural_zeros_alone() {
        // Three stored entries in a 2x2, so one position is structurally zero and is not visited.
        let mapped = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(m(), |x| x + 100.0);
        assert_eq!(
            mapped.values().len(),
            3,
            "the structural zero did not become 100"
        );
    }

    #[test]
    fn test_functor_identity() {
        let mapped = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(m(), |x| x);
        assert_eq!(mapped, m());
    }

    #[test]
    fn test_functor_composition() {
        let f = |x: f64| x + 1.0;
        let g = |x: f64| x * 2.0;
        let composed = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(m(), move |x| g(f(x)));
        let sequential = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(
            <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(m(), f),
            g,
        );
        assert_eq!(composed, sequential);
    }

    #[test]
    fn test_fold_visits_the_stored_entries_in_order() {
        // A sum cannot see the decision this fold documents. The fixture is a 2x2 storing
        // [1.0, 2.0, 3.0] with one structural zero, and a fold over the dense logical matrix
        // [1.0, 2.0, 0.0, 3.0] also totals 6.0. Record what was visited instead.
        let seen = <CsrMatrixWitness as Foldable<CsrMatrixWitness>>::fold(
            m(),
            Vec::new(),
            |mut acc, x| {
                acc.push(x);
                acc
            },
        );
        assert_eq!(
            seen,
            vec![1.0, 2.0, 3.0],
            "folds the three stored entries row-major, never the structural zero"
        );
    }

    #[test]
    fn test_fold_respects_the_initial_value() {
        let sum =
            <CsrMatrixWitness as Foldable<CsrMatrixWitness>>::fold(m(), 10.0, |acc, x| acc + x);
        assert_eq!(sum, 16.0);
    }

    #[test]
    fn test_pure_builds_the_one_by_one() {
        let p: CsrMatrix<f64> = CsrMatrixWitness::pure(42.0);
        assert_eq!(p.shape(), (1, 1));
        assert_eq!(p.get_value_at(0, 0), 42.0);
    }

    #[test]
    fn test_apply_applies_pointwise() {
        // `fmap` reaches the multi-entry path that `pure` alone cannot build, so the pointwise
        // behaviour is exercised on a matrix with more than one stored entry. The 1x1-against-1x1
        // form this test used to have could not fail: with one function and one value the
        // truncating `apply` has nothing to truncate, and only `values()` was asserted.
        let fns: CsrMatrix<fn(f64) -> f64> =
            <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(m(), |_| {
                (|x| x * 10.0) as fn(f64) -> f64
            });
        let applied = <CsrMatrixWitness as Applicative<CsrMatrixWitness>>::apply(fns, m());
        assert_eq!(applied.values(), &vec![10.0, 20.0, 30.0]);
        assert_eq!(applied.shape(), (2, 2));
    }

    #[test]
    fn test_apply_satisfies_the_applicative_identity_law() {
        // apply(pure(id), v) == v, with the CSR invariant checked alongside: the last row
        // pointer must equal the number of stored values.
        let idf: CsrMatrix<fn(f64) -> f64> = CsrMatrixWitness::pure((|x| x) as fn(f64) -> f64);
        let out = <CsrMatrixWitness as Applicative<CsrMatrixWitness>>::apply(idf, m());
        assert_eq!(out, m(), "applicative identity");
        assert_eq!(
            out.row_indices().last().copied(),
            Some(out.values().len()),
            "CSR row pointer must agree with values"
        );
    }

    #[test]
    fn test_extract_reads_the_first_stored_entry() {
        assert_eq!(
            <CsrMatrixWitness as CoMonad<CsrMatrixWitness>>::extract(&m()),
            1.0
        );
    }

    #[test]
    #[should_panic(expected = "stores nothing")]
    fn test_extract_panics_on_a_matrix_that_stores_nothing() {
        let empty: CsrMatrix<f64> = CsrMatrix::new();
        let _ = <CsrMatrixWitness as CoMonad<CsrMatrixWitness>>::extract(&empty);
    }

    #[test]
    fn test_extend_extract_is_the_identity() {
        // The comonad law. It holds because `extend` rotates each stored position to the front
        // before applying `f`, rather than handing `f` the whole container every time.
        let extended = <CsrMatrixWitness as CoMonad<CsrMatrixWitness>>::extend(
            &m(),
            <CsrMatrixWitness as CoMonad<CsrMatrixWitness>>::extract,
        );
        assert_eq!(extended, m());
    }
}
