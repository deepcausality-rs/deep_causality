/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, ViscousStress};

// =============================================================================
// `From<NewType> for raw` reverse conversions (uncovered before this block).
// These exercise the `impl From<Velocity3<R>> for [R; 3]` style impls that
// turn an invariant-bearing newtype back into its raw representation.
// =============================================================================

#[test]
fn test_viscous_stress_into_raw_matrix() {
    let m = [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let s = ViscousStress::<f64>::new(m).unwrap();
    let raw: [[f64; 3]; 3] = s.into();
    assert_eq!(raw, m);
}

// =============================================================================
// ViscousStress validation + accessors (fluids/mod.rs:593-595, 599-607)
// =============================================================================

#[test]
fn test_viscous_stress_new_asymmetric_error() {
    // fluids/mod.rs:592-596 — τ_ij != τ_ji must be rejected.
    let m = [[1.0, 2.0, 3.0], [9.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let s = ViscousStress::<f64>::new(m);
    assert!(s.is_err());
    match s.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("symmetric")),
        other => panic!("expected PhysicalInvariantBroken, got {other:?}"),
    }
}

#[test]
fn test_viscous_stress_new_unchecked_and_into_inner() {
    // fluids/mod.rs:599-601 (new_unchecked) and 605-607 (into_inner).
    let m = [[1.0, 9.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]]; // intentionally asymmetric
    let s = ViscousStress::<f64>::new_unchecked(m);
    assert_eq!(s.value(), &m);
    let inner = s.into_inner();
    assert_eq!(inner, m);
}
