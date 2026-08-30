/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::TopologicalInvariants;

fn build(betti: [usize; 4], a: f64, b: f64, h: f64) -> TopologicalInvariants<f64> {
    TopologicalInvariants::new(betti, a, b, h)
}

#[test]
fn test_equality_reflexivity() {
    let inv = build([1, 0, 0, 0], 1.0, 2.0, 0.5);
    assert_eq!(inv, inv);
}

#[test]
fn test_equality_symmetry_and_transitivity() {
    let a = build([1, 0, 0, 0], 1.0, 2.0, 0.5);
    let b = build([1, 0, 0, 0], 1.0, 2.0, 0.5);
    let c = build([1, 0, 0, 0], 1.0, 2.0, 0.5);
    assert_eq!(a, b);
    assert_eq!(b, a);
    assert_eq!(a, c);
}

#[test]
fn test_inequality_when_betti_differs() {
    let a = build([1, 0, 0, 0], 1.0, 2.0, 0.5);
    let b = build([2, 0, 0, 0], 1.0, 2.0, 0.5);
    assert_ne!(a, b);
}

#[test]
fn test_inequality_when_exact_norm_differs() {
    let a = build([1, 0, 0, 0], 1.0, 2.0, 0.5);
    let b = build([1, 0, 0, 0], 9.0, 2.0, 0.5);
    assert_ne!(a, b);
}

#[test]
fn test_inequality_when_co_exact_norm_differs() {
    let a = build([1, 0, 0, 0], 1.0, 2.0, 0.5);
    let b = build([1, 0, 0, 0], 1.0, 9.0, 0.5);
    assert_ne!(a, b);
}

#[test]
fn test_inequality_when_harmonic_norm_differs() {
    let a = build([1, 0, 0, 0], 1.0, 2.0, 0.5);
    let b = build([1, 0, 0, 0], 1.0, 2.0, 9.0);
    assert_ne!(a, b);
}

#[test]
fn test_equality_at_f32_precision() {
    let a: TopologicalInvariants<f32> =
        TopologicalInvariants::new([1, 0, 0, 0], 1.0_f32, 2.0_f32, 0.5_f32);
    let b: TopologicalInvariants<f32> =
        TopologicalInvariants::new([1, 0, 0, 0], 1.0_f32, 2.0_f32, 0.5_f32);
    assert_eq!(a, b);
}
