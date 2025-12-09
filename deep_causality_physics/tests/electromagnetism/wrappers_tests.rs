/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{lorentz_force, poynting_vector, magnetic_helicity_density};

// =============================================================================
// lorentz_force Wrapper Tests
// =============================================================================

#[test]
fn test_lorentz_force_wrapper_success() {
    let j = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();
    let b = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();

    let effect = lorentz_force(&j, &b);
    assert!(effect.is_ok());
}

// =============================================================================
// poynting_vector Wrapper Tests
// =============================================================================

#[test]
fn test_poynting_vector_wrapper_success() {
    // S = E × B
    let e = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();
    let b = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();

    let effect = poynting_vector(&e, &b);
    assert!(effect.is_ok());
}

// =============================================================================
// magnetic_helicity_density Wrapper Tests
// =============================================================================

#[test]
fn test_magnetic_helicity_density_wrapper_success() {
    // h = A · B
    let potential = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();
    let field = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();

    let effect = magnetic_helicity_density(&potential, &field);
    assert!(effect.is_ok());
}
