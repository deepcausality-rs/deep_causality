/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Definition 49 on the paper's own fixtures, Lorenz & Tull, arXiv:2602.16612, pp. 38–39.
//!
//! Example 54: low level `W → Z`, `Z → X`, `Z → Y`; high level `W → X`, `W → Y`, `W` an input.
//! With `π :: X ↦ {X}, Y ↦ {Y}, W ↦ {W}` the paper states `α(X) = {X, Z}`, `α(Y) = {Y, Z}`,
//! `α(W) = {W}`: simple, not extra-simple. With `W ↦ {W, Z}`, `α(N) = π(N)` for each `N`:
//! extra-simple. Example 55: low level `X → Y, Y', Z', Z`; high level `Y`, `Z` with `X` an input the
//! model discards, so `Y` and `Z` have no parents and nothing is blocked; the paper states
//! `X ∈ π(X) ∩ α(Y)`: not simple. Vertex numbering below: Example 54 low `X=0, Y=1, Z=2, W=3`,
//! high `X=0, Y=1, W=2`; Example 55 low `X=0, Y=1, Y'=2, Z'=3, Z=4`, high `X=0, Y=1, Z=2`.

use deep_causality_quantum::{
    AlignmentStructure, InducedDag, QuantumErrorEnum, StructureScope, check_alignment_structure,
};
use std::collections::BTreeSet;

fn set(v: &[usize]) -> BTreeSet<usize> {
    v.iter().copied().collect()
}

fn example_54() -> (InducedDag, InducedDag) {
    let mut low = InducedDag::new(4);
    low.add_edge(3, 2).add_edge(2, 0).add_edge(2, 1);
    let mut high = InducedDag::new(3);
    high.add_edge(2, 0).add_edge(2, 1);
    (low, high)
}

fn example_55() -> (InducedDag, InducedDag) {
    let mut low = InducedDag::new(5);
    low.add_edge(0, 1)
        .add_edge(0, 2)
        .add_edge(0, 3)
        .add_edge(0, 4);
    let high = InducedDag::new(3);
    (low, high)
}

#[test]
fn test_example_54_is_simple_and_not_extra_simple() {
    let (low, high) = example_54();
    let s: AlignmentStructure = check_alignment_structure(
        &low,
        &high,
        &[vec![0], vec![1], vec![3]],
        &[2],
        StructureScope::Equivalent,
    )
    .unwrap();
    assert_eq!(s.alpha[0], set(&[0, 2]), "α(X) = {{X, Z}}");
    assert_eq!(s.alpha[1], set(&[1, 2]), "α(Y) = {{Y, Z}}");
    assert_eq!(s.alpha[2], set(&[3]), "α(W) = {{W}}");
    assert!(s.simple);
    assert!(!s.extra_simple);
    assert_eq!(s.extra_simple_witness, Some((0, 1)));
    assert!(s.full, "W is an input, so fullness asks nothing");
    assert_eq!(s.scope, StructureScope::Equivalent);
    assert_eq!(s.pairs_examined, 6);
    let report = s.report::<f64>();
    assert_eq!(report.examined(), 6);
    assert!(
        !report.accepted(),
        "the extra-simplicity failure is a record"
    );
}

#[test]
fn test_example_54_alternative_partition_is_extra_simple() {
    let (low, high) = example_54();
    let s = check_alignment_structure(
        &low,
        &high,
        &[vec![0], vec![1], vec![3, 2]],
        &[2],
        StructureScope::Equivalent,
    )
    .unwrap();
    assert_eq!(s.alpha[0], set(&[0]));
    assert_eq!(s.alpha[1], set(&[1]));
    assert_eq!(s.alpha[2], set(&[2, 3]));
    assert!(s.simple && s.extra_simple && s.full);
    assert!(s.report::<f64>().accepted());
}

#[test]
fn test_example_55_is_not_simple() {
    let (low, high) = example_55();
    let s = check_alignment_structure(
        &low,
        &high,
        &[vec![0], vec![1, 2], vec![3, 4]],
        &[0],
        StructureScope::Equivalent,
    )
    .unwrap();
    assert_eq!(s.alpha[1], set(&[0, 1, 2]), "X ∈ α(Y)");
    assert!(!s.simple);
    assert_eq!(s.simple_witness, Some((1, 0)), "α(Y) meets π(X)");
    assert!(!s.extra_simple);
}

#[test]
fn test_fullness_fails_when_a_parent_block_does_not_reach() {
    // Low: A → B, C isolated. High: P → Q with π(P) = {A, C}, π(Q) = {B}; C ∉ inputs and C does
    // not reach B.
    let mut low = InducedDag::new(3);
    low.add_edge(0, 1);
    let mut high = InducedDag::new(2);
    high.add_edge(0, 1);
    let s = check_alignment_structure(
        &low,
        &high,
        &[vec![0, 2], vec![1]],
        &[],
        StructureScope::Necessary,
    )
    .unwrap();
    assert!(!s.full);
    assert_eq!(s.full_witness, Some((1, 2)));
    assert_eq!(s.scope, StructureScope::Necessary);
    // Declaring P an input lifts the question.
    let s = check_alignment_structure(
        &low,
        &high,
        &[vec![0, 2], vec![1]],
        &[0],
        StructureScope::Necessary,
    )
    .unwrap();
    assert!(s.full);
}

#[test]
fn test_partition_shape_errors() {
    let (low, high) = example_54();
    let wrong_count = check_alignment_structure(
        &low,
        &high,
        &[vec![0], vec![1]],
        &[],
        StructureScope::Equivalent,
    )
    .unwrap_err();
    assert!(matches!(
        wrong_count.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    let twice = check_alignment_structure(
        &low,
        &high,
        &[vec![0], vec![0], vec![3]],
        &[],
        StructureScope::Equivalent,
    )
    .unwrap_err();
    assert!(
        matches!(twice.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("two blocks"))
    );
    let out = check_alignment_structure(
        &low,
        &high,
        &[vec![0], vec![1], vec![9]],
        &[],
        StructureScope::Equivalent,
    )
    .unwrap_err();
    assert!(matches!(out.0, QuantumErrorEnum::DimensionMismatch(_)));
    let bad_input = check_alignment_structure(
        &low,
        &high,
        &[vec![0], vec![1], vec![3]],
        &[7],
        StructureScope::Equivalent,
    )
    .unwrap_err();
    assert!(matches!(
        bad_input.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

/// A block that repeats a vertex or holds none is not a block of a partition; a partition that
/// leaves a low-level vertex in no block is one, since the paper's `π` need not be onto.
#[test]
fn test_a_repeated_or_empty_block_is_refused_and_partial_cover_is_not() {
    let (low, high) = example_54();
    let repeated = check_alignment_structure(
        &low,
        &high,
        &[vec![0, 0], vec![1], vec![3]],
        &[2],
        StructureScope::Equivalent,
    )
    .unwrap_err();
    assert!(
        matches!(repeated.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("block 0") && m.contains("twice")),
        "{repeated:?}"
    );
    let empty = check_alignment_structure(
        &low,
        &high,
        &[vec![0], vec![], vec![3]],
        &[2],
        StructureScope::Equivalent,
    )
    .unwrap_err();
    assert!(
        matches!(empty.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("block 1") && m.contains("no low-level vertex")),
        "{empty:?}"
    );
    // Vertex Z = 2 lies in no block: Example 54's own partition.
    let partial = check_alignment_structure(
        &low,
        &high,
        &[vec![0], vec![1], vec![3]],
        &[2],
        StructureScope::Equivalent,
    )
    .unwrap();
    assert!(partial.simple);
}
