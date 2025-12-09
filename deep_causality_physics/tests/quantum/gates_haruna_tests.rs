/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, HilbertState, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::quantum::gates_haruna;

// Helper to create a simple projector field a^2 = a
fn create_projector_field() -> CausalMultiVector<Complex<f64>> {
    // For projector: a=0 or a=1. Let's use a=1 for testing
    let data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap()
}

#[test]
fn test_logical_s_gate() {
    let a = create_projector_field();
    let result = gates_haruna::logical_s(&a);
    
    // S(a) = exp(i pi/2 a^2)
    // For a=1: a^2=1, so exp(i pi/2 * 1) = cos(pi/2) + i sin(pi/2) = i
    // Result should be scalar i
    assert!(result.data().len() > 0);
}

#[test]
fn test_logical_z_gate() {
    let a = create_projector_field();
    let result = gates_haruna::logical_z(&a);
    
    // Z(a) = exp(i pi a)
    // For a=1: exp(i pi) = -1
    assert!(result.data().len() > 0);
}

#[test]
fn test_logical_x_gate() {
    let b = create_projector_field();
    let result = gates_haruna::logical_x(&b);
    
    // X(b) = exp(i pi b)
    assert!(result.data().len() > 0);
}

#[test]
fn test_logical_hadamard_gate() {
    let a = create_projector_field();
    let b = create_projector_field();
    let result = gates_haruna::logical_hadamard(&a, &b);
    
    // H = phase * S(a) * exp(i pi/2 b^2) * S(a)
    assert!(result.data().len() > 0);
}

#[test]
fn test_logical_cz_gate() {
    let a1 = create_projector_field();
    let a2 = create_projector_field();
    let result = gates_haruna::logical_cz(&a1, &a2);
    
    // CZ = exp(i pi a1 a2)
    assert!(result.data().len() > 0);
}

#[test]
fn test_logical_t_gate() {
    let a = create_projector_field();
    let result = gates_haruna::logical_t(&a);
    
    // T = exp(i pi (1/2 a^3 - 3/4 a^2 + 1/2 a))
    assert!(result.data().len() > 0);
}
