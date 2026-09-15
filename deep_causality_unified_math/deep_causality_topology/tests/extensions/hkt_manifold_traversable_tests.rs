/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Traversable` for `ManifoldWitness`, and why `DiagonalTraversable` is not here.
//!
//! `Traversable<F>: Functor<F> + Foldable<F>`, and the witness already had both, so `sequence` is
//! the conjunction of instances it carried separately.
//!
//! A `Manifold` validates that its data length equals the number of simplices in its complex, and
//! that its cursor is in bounds. `sequence` is one-in-one-out, so both survive by construction and
//! the complex, the metric and the cursor are carried across untouched — which is what the whole
//! structure is compared against below rather than the data tensor alone. A traversal that
//! rebuilt the payload correctly and dropped the geometry would pass a data-only assertion.
//!
//! `DiagonalTraversable` is absent, and `test_diagonal_seed_cannot_be_constructed` records why:
//! `sequence_zip` accumulates into `M::Type<F::Type<A>>`, growing each `F` one element at a time
//! from a caller-supplied seed. For a manifold that seed would have to start with zero data
//! against a non-empty complex, which `Manifold::new` rejects on both its length check and its
//! cursor check. The blocker is the manifold's invariant, not the trait.

use deep_causality_haft::{Functor, Traversable};
use deep_causality_haft::{OptionWitness, ResultWitness};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::manifold_cases;
use deep_causality_topology::{Manifold, ManifoldWitness, SimplicialComplex};

type W = ManifoldWitness<f64>;

const SEED: u64 = 0x71A_5EED;

// ---------------------------------------------------------------------------
// Success: the payload and the whole structure survive
// ---------------------------------------------------------------------------

#[test]
fn test_sequence_all_some_returns_the_input_unchanged() {
    // The identity traversal `sequence . fmap pure == pure`, asserted on the whole manifold so a
    // dropped complex, metric or cursor fails it.
    for case in manifold_cases(SEED) {
        let expected = case.value.clone();
        let wrapped = <W as Functor<W>>::fmap(case.value, Some);

        let out = W::sequence::<f64, OptionWitness>(wrapped);

        assert_eq!(
            out,
            Some(expected),
            "identity traversal changed {}",
            case.label
        );
    }
}

#[test]
fn test_sequence_preserves_the_complex_and_the_cursor() {
    for case in manifold_cases(SEED ^ 1) {
        let before = case.value.clone();
        let wrapped = <W as Functor<W>>::fmap(case.value, Ok::<f64, String>);

        let out = W::sequence::<f64, ResultWitness<String>>(wrapped).expect("all Ok must succeed");

        assert_eq!(
            out.data().as_slice(),
            before.data().as_slice(),
            "{}",
            case.label
        );
        assert_eq!(
            out.cursor(),
            before.cursor(),
            "cursor moved in {}",
            case.label
        );
        assert_eq!(out, before, "the structure changed in {}", case.label);
    }
}

// ---------------------------------------------------------------------------
// Failure: one bad cell collapses the traversal
// ---------------------------------------------------------------------------

#[test]
fn test_sequence_collapses_on_a_failure_at_any_position() {
    // A `None` at each index in turn, so a traversal that only inspects the head or the tail is
    // caught rather than passing on the one index it happens to read.
    for case in manifold_cases(SEED ^ 2) {
        let n = case.value.data().len();
        for bad in 0..n {
            let mut i = 0usize;
            let wrapped = <W as Functor<W>>::fmap(case.value.clone(), |x| {
                let keep = i != bad;
                i += 1;
                if keep { Some(x) } else { None }
            });

            assert_eq!(
                W::sequence::<f64, OptionWitness>(wrapped),
                None,
                "a None at index {bad} of {} did not collapse the traversal",
                case.label
            );
        }
    }
}

#[test]
fn test_sequence_reports_the_first_failure_in_cell_order() {
    // Two failures; the earlier cell wins, which pins the left-to-right fold direction.
    for case in manifold_cases(SEED ^ 3) {
        if case.value.data().len() < 4 {
            continue;
        }
        let mut i = 0usize;
        let wrapped = <W as Functor<W>>::fmap(case.value.clone(), |x| {
            let at = i;
            i += 1;
            match at {
                1 => Err("cell 1".to_string()),
                3 => Err("cell 3".to_string()),
                _ => Ok(x),
            }
        });

        assert_eq!(
            W::sequence::<f64, ResultWitness<String>>(wrapped),
            Err("cell 1".to_string()),
            "the earlier failure should surface in {}",
            case.label
        );
    }
}

// ---------------------------------------------------------------------------
// Why DiagonalTraversable is not implemented
// ---------------------------------------------------------------------------

#[test]
fn test_diagonal_seed_cannot_be_constructed() {
    // `sequence_zip` grows each `F` in the accumulator one element at a time from a seed the
    // caller supplies. For a manifold that seed must hold zero data against a non-empty complex.
    // `Manifold::new` rejects it — this is the blocker, recorded as a test so it is not
    // rediscovered as an oversight.
    let complex: SimplicialComplex<f64> = manifold_cases(SEED ^ 4)
        .into_iter()
        .next()
        .expect("at least one case")
        .value
        .complex()
        .clone();

    let empty: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();
    let seed = Manifold::new(complex, empty, 0);

    assert!(
        seed.is_err(),
        "an empty-data manifold is constructible, so DiagonalTraversable may be reachable after all"
    );
}
