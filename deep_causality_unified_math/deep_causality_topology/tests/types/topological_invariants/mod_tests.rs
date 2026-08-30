/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::TopologicalInvariants;

#[test]
fn test_new_constructs_with_prescribed_values() {
    let inv = TopologicalInvariants::<f64>::new([1, 2, 3, 4], 0.5, 1.0, 1.5);
    assert_eq!(inv.betti_numbers(), [1, 2, 3, 4]);
    assert_eq!(inv.exact_l2_norm(), 0.5);
    assert_eq!(inv.co_exact_l2_norm(), 1.0);
    assert_eq!(inv.harmonic_l2_norm(), 1.5);
}

#[test]
fn test_new_accepts_all_zero_betti_and_norms() {
    let inv = TopologicalInvariants::<f64>::new([0, 0, 0, 0], 0.0, 0.0, 0.0);
    assert_eq!(inv.betti_numbers(), [0, 0, 0, 0]);
    assert_eq!(inv.exact_l2_norm(), 0.0);
}

#[test]
fn test_new_accepts_arbitrary_large_betti_values() {
    let inv = TopologicalInvariants::<f64>::new([usize::MAX, 0, 0, 0], 0.0, 0.0, 0.0);
    assert_eq!(inv.betti_numbers()[0], usize::MAX);
}

#[test]
fn test_clone_is_independent() {
    let inv = TopologicalInvariants::<f64>::new([1, 0, 0, 0], 0.5, 1.0, 1.5);
    let cloned = inv.clone();
    assert_eq!(cloned.betti_numbers(), inv.betti_numbers());
    assert_eq!(cloned.exact_l2_norm(), inv.exact_l2_norm());
    assert_eq!(cloned.co_exact_l2_norm(), inv.co_exact_l2_norm());
    assert_eq!(cloned.harmonic_l2_norm(), inv.harmonic_l2_norm());
}

#[test]
fn test_new_at_f32_precision() {
    let inv: TopologicalInvariants<f32> =
        TopologicalInvariants::new([1, 0, 0, 0], 0.5_f32, 1.0_f32, 1.5_f32);
    assert_eq!(inv.betti_numbers(), [1, 0, 0, 0]);
}
