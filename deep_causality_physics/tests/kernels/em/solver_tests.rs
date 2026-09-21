/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_physics::{MaxwellSolver, PhysicsErrorEnum};

// ============================================================================
// Field Calculation Tests
// ============================================================================

#[test]
fn test_field_calculation_success() {
    let metric = Metric::Minkowski(4);

    // d = d/dx (e1) -> Index 2 (1<<1)
    let mut d_data = vec![0.0; 16];
    d_data[2] = 1.0;
    let d = CausalMultiVector::new(d_data, metric).unwrap();

    // A = A_y (e2) -> Index 4 (1<<2)
    let mut a_data = vec![0.0; 16];
    a_data[4] = 1.0;
    let a = CausalMultiVector::new(a_data, metric).unwrap();

    // F = d ^ A = e1 ^ e2 = e12
    let f = MaxwellSolver::calculate_field_tensor::<f64>(&d, &a).unwrap();

    let mag = f.squared_magnitude();
    assert!(
        (mag - 1.0).abs() < 1e-9,
        "Expected magnitude 1.0, got {}",
        mag
    );
}

#[test]
fn test_field_calculation_metric_mismatch() {
    let d = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let a = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();

    let result = MaxwellSolver::calculate_field_tensor::<f64>(&d, &a);
    match result {
        Err(e) => match e.0 {
            PhysicsErrorEnum::DimensionMismatch(_) => {}
            _ => panic!("Expected DimensionMismatch, got {:?}", e),
        },
        Ok(_) => panic!("Should fail on metric mismatch"),
    }
}

#[test]
fn test_field_calculation_non_finite_input() {
    let metric = Metric::Minkowski(4);
    let mut d_data = vec![0.0; 16];
    d_data[2] = f64::INFINITY; // Infinite derivative
    let d = CausalMultiVector::new(d_data, metric).unwrap();

    let mut a_data = vec![0.0; 16];
    a_data[4] = 1.0; // Finite potential
    let a = CausalMultiVector::new(a_data, metric).unwrap();

    // If input is infinite, geometric product will be infinite/NaN
    let result = MaxwellSolver::calculate_field_tensor::<f64>(&d, &a);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::NumericalInstability { .. }
        ),
        "expected a NumericalInstability refusal"
    );
}

// ============================================================================
// Potential Divergence Tests
// ============================================================================

#[test]
fn test_potential_divergence_success_zero() {
    let metric = Metric::Euclidean(3);
    // Orthogonal: d = e1, A = e2 -> d.A = 0
    let mut d_data = vec![0.0; 8];
    d_data[1] = 1.0; // e1
    let d = CausalMultiVector::new(d_data, metric).unwrap();

    let mut a_data = vec![0.0; 8];
    a_data[2] = 1.0; // e2
    let a = CausalMultiVector::new(a_data, metric).unwrap();

    let div = MaxwellSolver::calculate_potential_divergence::<f64>(&d, &a).unwrap();
    assert_eq!(div, 0.0);
}

#[test]
fn test_potential_divergence_non_zero() {
    let metric = Metric::Euclidean(3);
    // Parallel: d = e1, A = e1 -> d.A = 1
    let mut d_data = vec![0.0; 8];
    d_data[1] = 1.0;
    let d = CausalMultiVector::new(d_data, metric).unwrap();

    let mut a_data = vec![0.0; 8];
    a_data[1] = 1.0;
    let a = CausalMultiVector::new(a_data, metric).unwrap();

    let div = MaxwellSolver::calculate_potential_divergence::<f64>(&d, &a).unwrap();
    assert!((div - 1.0).abs() < 1e-9);
}

#[test]
fn test_potential_divergence_non_finite_scalar_result() {
    // Both inputs are *pure grade-1* (only e1 populated), so they pass the
    // pure-grade validation, but their inner product e1·e1 = MAX*MAX overflows
    // to +inf, tripping the non-finite scalar guard at solver.rs:63-66.
    let metric = Metric::Euclidean(3);
    let mut d_data = vec![0.0; 8];
    d_data[1] = f64::MAX; // e1 only -> pure grade 1
    let d = CausalMultiVector::new(d_data, metric).unwrap();

    let mut a_data = vec![0.0; 8];
    a_data[1] = f64::MAX; // e1 only -> pure grade 1
    let a = CausalMultiVector::new(a_data, metric).unwrap();

    match MaxwellSolver::calculate_potential_divergence::<f64>(&d, &a) {
        Err(e) => match e.0 {
            PhysicsErrorEnum::NumericalInstability(_) => {}
            _ => panic!("Expected NumericalInstability, got {:?}", e),
        },
        Ok(v) => panic!("Expected non-finite divergence error, got {}", v),
    }
}

#[test]
fn test_potential_divergence_metric_mismatch() {
    let d = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let a = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    assert!(
        matches!(
            MaxwellSolver::calculate_potential_divergence::<f64>(&d, &a)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_potential_divergence_non_finite() {
    let metric = Metric::Minkowski(4);
    let mut d_data = vec![0.0; 16];
    d_data[0] = f64::NAN;
    let d = CausalMultiVector::new(d_data, metric).unwrap();
    let a = CausalMultiVector::new(vec![1.0; 16], metric).unwrap();

    match MaxwellSolver::calculate_potential_divergence::<f64>(&d, &a) {
        Err(e) => match e.0 {
            // Either grade validation catches it as non-pure-vector or finiteness check catches NaN
            PhysicsErrorEnum::NumericalInstability(_)
            | PhysicsErrorEnum::PhysicalInvariantBroken(_) => {}
            _ => panic!(
                "Expected NumericalInstability or PhysicalInvariantBroken, got {:?}",
                e
            ),
        },
        Ok(_) => panic!("Should fail"),
    }
}

// ============================================================================
// Current Density Tests
// ============================================================================

#[test]
fn test_current_density_success() {
    let metric = Metric::Minkowski(4);
    // J = d . F
    // Let d = e1 (index 2)
    // Let F = e12 (index 6, which is 1010?? No 110. 2^4=6).
    // e1 . (e1 ^ e2) = e2.
    // So J should be e2 (index 4).

    let mut d_data = vec![0.0; 16];
    d_data[2] = 1.0;
    let d = CausalMultiVector::new(d_data, metric).unwrap();

    let mut f_data = vec![0.0; 16];
    f_data[6] = 1.0; // e12
    let f = CausalMultiVector::new(f_data, metric).unwrap();

    let j = MaxwellSolver::calculate_current_density::<f64>(&d, &f).unwrap();

    // J is the grade-1 part of `d.inner_product(F)`, the left contraction, with d = e1 and
    // F = e12:
    //
    //     e1 _| (e1 ^ e2) = (e1 . e1) e2 - (e1 . e2) e1 = (e1 . e1) e2
    //
    // `Metric::Minkowski` is west coast, (+ - - -): e0 squares to +1 and every spatial basis
    // vector squares to -1. So e1 . e1 = -1 and the answer is -e2, on blade index 4.
    //
    // The assertion used to accept either sign, with a comment reasoning from the east-coast
    // signature (where e1 . e1 = +1 and the answer would be +e2). Under the signature this crate
    // actually uses, -1 is the only correct value.
    let d_out: &[f64] = j.data();
    assert!((d_out[4] + 1.0).abs() < 1e-9, "e2 component = {}", d_out[4]);
    for (i, v) in d_out.iter().enumerate() {
        if i != 4 {
            assert!(v.abs() < 1e-9, "blade {i} should vanish, got {v}");
        }
    }
}

#[test]
fn test_current_density_mismatch() {
    let d = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let f = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    assert!(
        matches!(
            MaxwellSolver::calculate_current_density::<f64>(&d, &f)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

// ============================================================================
// Poynting Flux Tests
// ============================================================================

#[test]
fn test_poynting_flux_success() {
    let metric = Metric::Euclidean(3);
    // S = E x B
    // E = e1 (index 1 in 3D?)
    // B = e2 (index 2 in 3D)
    // S = e1 ^ e2 = e12 (index 3 in 3D bitmask: 1|2=3)

    let mut e_data = vec![0.0; 8];
    e_data[1] = 1.0;
    let e = CausalMultiVector::new(e_data, metric).unwrap();

    let mut b_data = vec![0.0; 8];
    b_data[2] = 1.0;
    let b = CausalMultiVector::new(b_data, metric).unwrap();

    let s = MaxwellSolver::calculate_poynting_flux::<f64>(&e, &b).unwrap();

    // Check index 3
    let val = s.data()[3];
    assert!((val - 1.0).abs() < 1e-9);
}

#[test]
fn test_poynting_flux_mismatch() {
    let e = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let b = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    assert!(
        matches!(
            MaxwellSolver::calculate_poynting_flux::<f64>(&e, &b)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}
