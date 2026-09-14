/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The diagonal traversal on [`DenseVectorWitness`].
//!
//! The same operation the tensor witness carries, on a carrier with no shape to restore. Every
//! assertion is on values: a count cannot separate "paired by index" from "paired by index, off
//! by one".

use deep_causality_haft::DiagonalTraversable;
use deep_causality_linear::{DenseVector, DenseVectorWitness, ZipDenseVectorWitness};

const DRAWS: usize = 50;

/// Four slots, slot `k` holding the run `100k .. 100k + DRAWS`.
fn field() -> DenseVector<DenseVector<i32>> {
    DenseVector::from_vec(
        (0..4)
            .map(|k| DenseVector::from_vec((0..DRAWS).map(|i| (k * 100 + i) as i32).collect()))
            .collect(),
    )
}

fn seed(n: usize) -> DenseVector<DenseVector<i32>> {
    DenseVector::from_vec((0..n).map(|_| DenseVector::from_vec(Vec::new())).collect())
}

#[test]
fn the_diagonal_pairs_index_with_index() {
    let out = DenseVectorWitness::sequence_zip::<i32, ZipDenseVectorWitness>(field(), seed(DRAWS));

    assert_eq!(out.len(), DRAWS, "one vector per draw");
    for (i, v) in out.as_slice().iter().enumerate() {
        let want: Vec<i32> = (0..4).map(|k| (k * 100 + i) as i32).collect();
        assert_eq!(
            v.as_slice(),
            want.as_slice(),
            "vector {i} does not hold draw {i} of every slot"
        );
    }
}

#[test]
fn an_empty_structure_returns_the_seed() {
    let empty: DenseVector<DenseVector<i32>> = DenseVector::from_vec(Vec::new());
    let out = DenseVectorWitness::sequence_zip::<i32, ZipDenseVectorWitness>(empty, seed(7));
    assert_eq!(out.len(), 7);
    assert!(out.as_slice().iter().all(|v| v.as_slice().is_empty()));
}

#[test]
fn ragged_slots_truncate_the_ensemble_and_not_the_structure() {
    let ragged = DenseVector::from_vec(
        (0..4)
            .map(|k| {
                let n = if k == 2 { 12 } else { DRAWS };
                DenseVector::from_vec((0..n).map(|i| (k * 100 + i) as i32).collect())
            })
            .collect(),
    );

    let out = DenseVectorWitness::sequence_zip::<i32, ZipDenseVectorWitness>(ragged, seed(DRAWS));

    assert_eq!(out.len(), 12, "the ensemble truncated to the shortest slot");
    for (i, v) in out.as_slice().iter().enumerate() {
        assert_eq!(v.len(), 4, "vector {i} lost a slot to the truncation");
        let want: Vec<i32> = (0..4).map(|k| (k * 100 + i) as i32).collect();
        assert_eq!(v.as_slice(), want.as_slice(), "vector {i} mispaired");
    }
}

#[test]
fn a_seed_shorter_than_the_slots_bounds_the_ensemble() {
    let out = DenseVectorWitness::sequence_zip::<i32, ZipDenseVectorWitness>(field(), seed(4));
    assert_eq!(out.len(), 4);
    for (i, v) in out.as_slice().iter().enumerate() {
        let want: Vec<i32> = (0..4).map(|k| (k * 100 + i) as i32).collect();
        assert_eq!(v.as_slice(), want.as_slice(), "vector {i} mispaired");
    }
}
