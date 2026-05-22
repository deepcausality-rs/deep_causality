/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::TopologicalInvariants;

#[test]
fn test_display_format_contains_all_fields() {
    let inv = TopologicalInvariants::<f64>::new([1, 2, 3, 4], 0.5, 1.0, 1.5);
    let s = format!("{}", inv);
    assert!(s.contains("betti=[1,2,3,4]"));
    assert!(s.contains("exact_l2=0.5"));
    assert!(s.contains("co_exact_l2=1"));
    assert!(s.contains("harmonic_l2=1.5"));
}

#[test]
fn test_display_format_with_zero_betti() {
    let inv = TopologicalInvariants::<f64>::new([0, 0, 0, 0], 0.0, 0.0, 0.0);
    let s = format!("{}", inv);
    assert!(s.contains("betti=[0,0,0,0]"));
}

#[test]
fn test_debug_format_is_non_empty() {
    let inv = TopologicalInvariants::<f64>::new([1, 0, 0, 0], 1.0, 1.0, 1.0);
    let s = format!("{:?}", inv);
    assert!(s.contains("TopologicalInvariants"));
}

#[test]
fn test_display_format_at_f32_precision() {
    let inv: TopologicalInvariants<f32> =
        TopologicalInvariants::new([2, 0, 0, 0], 0.25_f32, 0.5_f32, 0.75_f32);
    let s = format!("{}", inv);
    assert!(s.contains("betti=[2,0,0,0]"));
    assert!(s.contains("exact_l2=0.25"));
}
