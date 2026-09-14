/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Building a [`CausalTensor`] from a sequence, on [`CausalTensorWitness`].
//!
//! The direction `Foldable` cannot run. Unlike the vector carrier this one has a shape, so the
//! rank the operation produces is itself an assertion rather than an implementation detail.

use deep_causality_haft::{Collectable, Foldable, Pure};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

#[test]
fn values_land_in_iteration_order() {
    let out: CausalTensor<i32> = CausalTensorWitness::collect([10, 20, 30, 40]);

    assert_eq!(
        out.as_slice(),
        [10, 20, 30, 40].as_slice(),
        "the i-th value must occupy the i-th slot"
    );
}

#[test]
fn the_result_is_rank_one_with_the_sequence_length() {
    let out: CausalTensor<i32> = CausalTensorWitness::collect([1, 2, 3, 4, 5]);

    assert_eq!(out.shape(), [5].as_slice(), "rank 1, extent 5");
}

#[test]
fn one_value_is_a_run_of_one_not_a_scalar() {
    // This is where `collect` and `pure` differ, and the difference is deliberate: `pure` builds
    // the rank-0 scalar, while a sequence of one value is a run of length one.
    let collected: CausalTensor<i32> = CausalTensorWitness::collect([42]);
    let pured: CausalTensor<i32> = CausalTensorWitness::pure(42);

    assert_eq!(collected.shape(), [1].as_slice(), "collect gives rank 1");
    assert_eq!(pured.shape(), [0usize; 0].as_slice(), "pure gives rank 0");
    assert_eq!(collected.as_slice(), pured.as_slice(), "same one element");
}

#[test]
fn the_empty_sequence_gives_the_empty_tensor() {
    let out: CausalTensor<i32> = CausalTensorWitness::collect(Vec::new());

    assert_eq!(out.shape(), [0].as_slice(), "rank 1, extent 0");
    assert_eq!(
        CausalTensorWitness::fold(out, 7, |acc, x| acc + x),
        7,
        "folding the empty structure must return the initial accumulator unchanged"
    );
}

#[test]
fn collect_then_fold_equals_folding_the_sequence() {
    // The round-trip law, on a fold that is not commutative, so an order error cannot cancel out.
    let xs = [1, 2, 3, 4, 5];

    let through_structure =
        CausalTensorWitness::fold(CausalTensorWitness::collect(xs), 0, |acc, x| acc * 2 + x);
    let direct = xs.into_iter().fold(0, |acc, x| acc * 2 + x);

    assert_eq!(through_structure, direct);
    assert_eq!(direct, 57, "pinned so a change to either side is visible");
}

#[test]
fn a_lazy_sequence_needs_no_intermediate_vec() {
    let out: CausalTensor<i32> = CausalTensorWitness::collect((0..4).map(|i| i * i));

    assert_eq!(out.as_slice(), [0, 1, 4, 9].as_slice());
    assert_eq!(out.shape(), [4].as_slice());
}

#[test]
fn the_element_type_carries_no_bound() {
    let out: CausalTensor<&str> = CausalTensorWitness::collect(["a", "b", "c"]);

    assert_eq!(out.as_slice(), ["a", "b", "c"].as_slice());
    assert_eq!(out.shape(), [3].as_slice());
}
