/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Traversable` for `CausalMultiVectorWitness`.
//!
//! The witness has no `Monad`, because `bind`'s continuation may hand back a different `Metric`
//! than the input carries and the two identity laws then want opposite choices. `sequence` never
//! faces that choice: it is one-in-one-out, so the input's metric is the only answer and the
//! `2^dim` coefficient count cannot change. These tests pin both halves — the values and their
//! order, and the metric that fixes their number.
//!
//! Every case uses distinct non-zero coefficients across several algebras. A traversal that
//! returned the right values in the wrong order, or the right count under the wrong metric, is
//! invisible to a test built from zeros or from one blade.

use deep_causality_haft::{Foldable, Functor, OptionWitness, ResultWitness, Traversable};
use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness as W};

/// The algebras under test, each with the coefficient count `2^dim` its metric admits.
fn metrics() -> Vec<(Metric, usize)> {
    vec![
        (Metric::Euclidean(0), 1),
        (Metric::Euclidean(1), 2),
        (Metric::Euclidean(2), 4),
        (Metric::Euclidean(3), 8),
        (Metric::Minkowski(4), 16),
    ]
}

/// Distinct, non-zero, and not equal to their own index, so a shift or a reversal shows.
fn coefficients(len: usize) -> Vec<f64> {
    (0..len).map(|i| (i as f64) * 3.0 + 1.5).collect()
}

// ---------------------------------------------------------------------------
// Success: values, order and metric all survive
// ---------------------------------------------------------------------------

#[test]
fn test_sequence_option_all_some_preserves_values_order_and_metric() {
    for (metric, len) in metrics() {
        let data = coefficients(len);
        let mv = CausalMultiVector::new(data.clone(), metric).unwrap();
        let wrapped = W::fmap(mv, Some);

        let out = W::sequence::<f64, OptionWitness>(wrapped).expect("all Some must succeed");

        assert_eq!(
            out.data(),
            &data,
            "values or their order changed at {metric:?}"
        );
        assert_eq!(out.metric(), metric, "metric changed at {metric:?}");
        assert_eq!(
            out.data().len(),
            len,
            "coefficient count changed at {metric:?}"
        );
    }
}

#[test]
fn test_sequence_result_all_ok_preserves_values_order_and_metric() {
    for (metric, len) in metrics() {
        let data = coefficients(len);
        let mv = CausalMultiVector::new(data.clone(), metric).unwrap();
        let wrapped = W::fmap(mv, Ok::<f64, String>);

        let out = W::sequence::<f64, ResultWitness<String>>(wrapped).expect("all Ok must succeed");

        assert_eq!(out.data(), &data);
        assert_eq!(out.metric(), metric);
    }
}

// ---------------------------------------------------------------------------
// Failure: one bad element collapses the traversal
// ---------------------------------------------------------------------------

#[test]
fn test_sequence_option_single_none_collapses_the_whole_traversal() {
    // A failure at *each* position in turn, so a traversal that only inspects the head or the
    // tail is caught rather than passing on the one index it happens to read.
    let (metric, len) = (Metric::Euclidean(3), 8);
    for bad in 0..len {
        let wrapped = CausalMultiVector::new(
            (0..len)
                .map(|i| if i == bad { None } else { Some(i as f64) })
                .collect(),
            metric,
        )
        .unwrap();

        assert_eq!(
            W::sequence::<f64, OptionWitness>(wrapped),
            None,
            "a None at index {bad} did not collapse the traversal"
        );
    }
}

#[test]
fn test_sequence_result_reports_the_first_failure_in_blade_order() {
    // Two failures. Which one surfaces is the ordering guarantee: the fold runs left to right,
    // so the earlier blade wins. Swap the fold's direction and this test fails.
    let wrapped = CausalMultiVector::new(
        vec![
            Ok(1.0),
            Err("blade 1".to_string()),
            Ok(3.0),
            Err("blade 3".to_string()),
        ],
        Metric::Euclidean(2),
    )
    .unwrap();

    assert_eq!(
        W::sequence::<f64, ResultWitness<String>>(wrapped),
        Err("blade 1".to_string())
    );
}

// ---------------------------------------------------------------------------
// Laws
// ---------------------------------------------------------------------------

#[test]
fn test_sequence_identity_law_pure_traversal_returns_the_input() {
    // `sequence . fmap pure == pure` — the identity traversal. It pins that `sequence` adds no
    // effect of its own, over every metric rather than over one.
    for (metric, len) in metrics() {
        let mv = CausalMultiVector::new(coefficients(len), metric).unwrap();
        let expected = mv.clone();

        let out = W::sequence::<f64, OptionWitness>(W::fmap(mv, Some));

        assert_eq!(
            out,
            Some(expected),
            "identity traversal changed the value at {metric:?}"
        );
    }
}

#[test]
fn test_sequence_agrees_with_fold_on_the_element_order() {
    // Traversable's two supertraits must not disagree: the order `sequence` rebuilds in is the
    // order `fold` visits in. Subtraction is non-commutative, so a permutation changes the result.
    for (metric, len) in metrics() {
        let data = coefficients(len);
        let mv = CausalMultiVector::new(data.clone(), metric).unwrap();

        let via_sequence = W::sequence::<f64, OptionWitness>(W::fmap(mv, Some)).unwrap();
        let folded_after = W::fold(via_sequence, 0.0, |acc, x| acc - x);
        let folded_direct = W::fold(
            CausalMultiVector::new(data, metric).unwrap(),
            0.0,
            |acc, x| acc - x,
        );

        assert_eq!(folded_after, folded_direct, "order disagreed at {metric:?}");
    }
}

// ---------------------------------------------------------------------------
// Edges
// ---------------------------------------------------------------------------

#[test]
fn test_sequence_on_the_one_coefficient_algebra() {
    // `Cl(0)` has exactly one blade. The accumulator starts empty, so a fold that mishandles the
    // first step degenerates precisely here and nowhere else.
    let mv = CausalMultiVector::new(vec![7.25], Metric::Euclidean(0)).unwrap();

    let out = W::sequence::<f64, OptionWitness>(W::fmap(mv, Some)).unwrap();

    assert_eq!(out.data(), &vec![7.25]);
    assert_eq!(out.metric(), Metric::Euclidean(0));
}

#[test]
fn test_sequence_changes_the_element_type() {
    // `sequence` is generic in `A`; the rebuilt multivector carries the inner type, not the outer.
    let mv = CausalMultiVector::new(vec![1u8, 2, 3, 4], Metric::Euclidean(2)).unwrap();

    let out: CausalMultiVector<u8> = W::sequence::<u8, OptionWitness>(W::fmap(mv, Some)).unwrap();

    assert_eq!(out.data(), &vec![1u8, 2, 3, 4]);
    assert_eq!(out.metric(), Metric::Euclidean(2));
}
