/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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
    let f = MaxwellSolver::calculate_field_tensor(&d, &a).unwrap();

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

    let result = MaxwellSolver::calculate_field_tensor(&d, &a);
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
    let result = MaxwellSolver::calculate_field_tensor(&d, &a);
    assert!(
        result.is_err(),
        "Should detect non-finite result via validate_finiteness"
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

    let div = MaxwellSolver::calculate_potential_divergence(&d, &a).unwrap();
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

    let div = MaxwellSolver::calculate_potential_divergence(&d, &a).unwrap();
    assert!((div - 1.0).abs() < 1e-9);
}

#[test]
fn test_potential_divergence_metric_mismatch() {
    let d = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let a = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    assert!(MaxwellSolver::calculate_potential_divergence(&d, &a).is_err());
}

#[test]
fn test_potential_divergence_non_finite() {
    let metric = Metric::Minkowski(4);
    let mut d_data = vec![0.0; 16];
    d_data[0] = f64::NAN;
    let d = CausalMultiVector::new(d_data, metric).unwrap();
    let a = CausalMultiVector::new(vec![1.0; 16], metric).unwrap();

    match MaxwellSolver::calculate_potential_divergence(&d, &a) {
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

    let j = MaxwellSolver::calculate_current_density(&d, &f).unwrap();

    // Check J is e2 (index 4)
    let val = j.data()[4];
    assert!(
        (val - 1.0).abs() < 1e-9 || (val + 1.0).abs() < 1e-9,
        "Result: {:?}",
        j.data()
    );
    // Note: sign depends on metric signature/contraction order.
    // e1 . (e1 e2) = (e1 . e1) e2 - (e1 . e2) e1 = (1) e2 - 0 = e2. (Minkowski e1 squared is +1?)
    // If (- + + +), e0^2=-1, e1^2=1. Correct.
}

#[test]
fn test_current_density_mismatch() {
    let d = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let f = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    assert!(MaxwellSolver::calculate_current_density(&d, &f).is_err());
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

    let s = MaxwellSolver::calculate_poynting_flux(&e, &b).unwrap();

    // Check index 3
    let val = s.data()[3];
    assert!((val - 1.0).abs() < 1e-9);
}

#[test]
fn test_poynting_flux_mismatch() {
    let e = CausalMultiVector::new(vec![0.0; 16], Metric::Minkowski(4)).unwrap();
    let b = CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap();
    assert!(MaxwellSolver::calculate_poynting_flux(&e, &b).is_err());
}
