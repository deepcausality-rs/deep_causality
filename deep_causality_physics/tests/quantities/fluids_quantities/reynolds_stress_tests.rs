/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, ReynoldsStress};

// =============================================================================
// `From<NewType> for raw` reverse conversions (uncovered before this block).
// These exercise the `impl From<Velocity3<R>> for [R; 3]` style impls that
// turn an invariant-bearing newtype back into its raw representation.
// =============================================================================

#[test]
fn test_reynolds_stress_into_raw_matrix() {
    let m = [[1.0, 0.5, 0.0], [0.5, 2.0, 0.0], [0.0, 0.0, 1.5]];
    let r = ReynoldsStress::<f64>::new(m).unwrap();
    let raw: [[f64; 3]; 3] = r.into();
    assert_eq!(raw, m);
}

// =============================================================================
// ReynoldsStress validation + accessors (fluids/mod.rs:634-636, 639-641, 651-653)
// =============================================================================

#[test]
fn test_reynolds_stress_new_non_finite_error() {
    // fluids/mod.rs:633-636 — non-finite components rejected.
    let m = [[f64::NAN, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let r = ReynoldsStress::<f64>::new(m);
    assert!(r.is_err());
    match r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        other => panic!("expected PhysicalInvariantBroken, got {other:?}"),
    }
}

#[test]
fn test_reynolds_stress_new_asymmetric_error() {
    // fluids/mod.rs:638-641 — R_ij != R_ji must be rejected.
    let m = [[1.0, 0.5, 0.0], [9.0, 2.0, 0.0], [0.0, 0.0, 1.5]];
    let r = ReynoldsStress::<f64>::new(m);
    assert!(r.is_err());
    match r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("symmetric")),
        other => panic!("expected PhysicalInvariantBroken, got {other:?}"),
    }
}

#[test]
fn test_reynolds_stress_new_unchecked_and_into_inner() {
    // fluids/mod.rs:645-647 (new_unchecked) and 651-653 (into_inner).
    let m = [[1.0, 9.0, 0.0], [0.5, 2.0, 0.0], [0.0, 0.0, 1.5]]; // intentionally asymmetric
    let r = ReynoldsStress::<f64>::new_unchecked(m);
    assert_eq!(r.value(), &m);
    let inner = r.into_inner();
    assert_eq!(inner, m);
}
