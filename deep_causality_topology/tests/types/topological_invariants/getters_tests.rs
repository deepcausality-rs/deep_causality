/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::TopologicalInvariants;

fn sample() -> TopologicalInvariants<f64> {
    TopologicalInvariants::new([1, 3, 3, 1], 1.0, 2.0, 0.5)
}

#[test]
fn test_betti_numbers_returned_by_value() {
    let inv = sample();
    let b: [usize; 4] = inv.betti_numbers();
    assert_eq!(b, [1, 3, 3, 1]);
}

#[test]
fn test_exact_l2_norm_returns_stored_value() {
    let inv = sample();
    assert_eq!(inv.exact_l2_norm(), 1.0);
}

#[test]
fn test_co_exact_l2_norm_returns_stored_value() {
    let inv = sample();
    assert_eq!(inv.co_exact_l2_norm(), 2.0);
}

#[test]
fn test_harmonic_l2_norm_returns_stored_value() {
    let inv = sample();
    assert_eq!(inv.harmonic_l2_norm(), 0.5);
}

#[test]
fn test_getters_are_idempotent_across_multiple_calls() {
    let inv = sample();
    assert_eq!(inv.betti_numbers(), inv.betti_numbers());
    assert_eq!(inv.exact_l2_norm(), inv.exact_l2_norm());
}
