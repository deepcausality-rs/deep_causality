/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Building a [`DenseVector`] from a sequence, on [`DenseVectorWitness`].
//!
//! The direction `Foldable` cannot run. Every assertion is on values rather than on a length: a
//! count cannot separate "in order" from "reversed", and both have the same length.

use deep_causality_haft::{Collectable, Foldable};
use deep_causality_linear::{DenseVector, DenseVectorWitness};

#[test]
fn values_land_in_iteration_order() {
    let out: DenseVector<i32> = DenseVectorWitness::collect([10, 20, 30, 40]);

    assert_eq!(
        out.as_slice(),
        [10, 20, 30, 40].as_slice(),
        "the i-th value must occupy the i-th slot"
    );
}

#[test]
fn the_empty_sequence_gives_the_empty_vector() {
    let out: DenseVector<i32> = DenseVectorWitness::collect(Vec::new());

    assert_eq!(out.len(), 0);
    assert_eq!(
        DenseVectorWitness::fold(out, 7, |acc, x| acc + x),
        7,
        "folding the empty structure must return the initial accumulator unchanged"
    );
}

#[test]
fn collect_then_fold_equals_folding_the_sequence() {
    // The round-trip law, on a fold that is not commutative, so an order error cannot cancel out.
    let xs = [1, 2, 3, 4, 5];

    let through_structure =
        DenseVectorWitness::fold(DenseVectorWitness::collect(xs), 0, |acc, x| acc * 2 + x);
    let direct = xs.into_iter().fold(0, |acc, x| acc * 2 + x);

    assert_eq!(through_structure, direct);
    assert_eq!(direct, 57, "pinned so a change to either side is visible");
}

#[test]
fn a_lazy_sequence_needs_no_intermediate_vec() {
    // `IntoIterator` rather than `Vec` is the point: a caller producing values one at a time does
    // not have to collect them first.
    let out: DenseVector<i32> = DenseVectorWitness::collect((0..4).map(|i| i * i));

    assert_eq!(out.as_slice(), [0, 1, 4, 9].as_slice());
}

#[test]
fn one_value_is_a_run_of_one() {
    let out: DenseVector<i32> = DenseVectorWitness::collect([42]);

    assert_eq!(out.as_slice(), [42].as_slice());
    assert_eq!(out.len(), 1);
}

#[test]
fn the_element_type_carries_no_bound() {
    // The witness does no arithmetic, so a vector of labels collects as readily as one of numbers.
    let out: DenseVector<&str> = DenseVectorWitness::collect(["a", "b", "c"]);

    assert_eq!(out.as_slice(), ["a", "b", "c"].as_slice());
}
