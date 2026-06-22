/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// ---------------------------------------------------------------------------
// Wall-bounded coordinate projections: constrain_edges / with_lift
// ---------------------------------------------------------------------------

use deep_causality_physics::utils_tests::{random_cochain, unit_manifold};
use deep_causality_physics::{SolenoidalField, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::ChainComplex;

#[test]
fn constrain_edges_zeroes_listed_edges() {
    let manifold = unit_manifold::<f64>(4);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(random_cochain::<f64>(n1, 91), vec![n1]).unwrap();
    let velocity = VelocityOneForm::new(raw, &manifold).unwrap();
    let (field, _) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();

    let edges = [0usize, 2, 5];
    let before: Vec<f64> = field.as_one_form().as_slice().to_vec();
    let constrained = field.constrain_edges(&edges);
    let after = constrained.as_one_form().as_slice();

    for &e in &edges {
        assert_eq!(after[e], 0.0, "edge {e} must be zeroed");
    }
    // Every other edge is untouched.
    for i in 0..n1 {
        if !edges.contains(&i) {
            assert_eq!(after[i], before[i], "edge {i} must be unchanged");
        }
    }
    assert_eq!(constrained.len(), n1);
}

#[test]
fn constrain_edges_empty_is_noop() {
    let manifold = unit_manifold::<f64>(3);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(random_cochain::<f64>(n1, 93), vec![n1]).unwrap();
    let velocity = VelocityOneForm::new(raw, &manifold).unwrap();
    let (field, _) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();

    let snapshot = field.clone();
    let result = field.constrain_edges(&[]);
    assert_eq!(
        result, snapshot,
        "empty constrain_edges is a bit-exact no-op"
    );
}

#[test]
fn with_lift_assigns_prescribed_values() {
    let manifold = unit_manifold::<f64>(4);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(random_cochain::<f64>(n1, 95), vec![n1]).unwrap();
    let velocity = VelocityOneForm::new(raw, &manifold).unwrap();
    let (field, _) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();

    let before: Vec<f64> = field.as_one_form().as_slice().to_vec();
    let lift = [(1usize, 0.25_f64), (3usize, -1.5_f64)];
    let lifted = field.with_lift(&lift);
    let after = lifted.as_one_form().as_slice();

    assert_eq!(after[1], 0.25);
    assert_eq!(after[3], -1.5);
    for i in 0..n1 {
        if i != 1 && i != 3 {
            assert_eq!(after[i], before[i], "edge {i} must be unchanged");
        }
    }
}

#[test]
fn with_lift_empty_is_noop() {
    let manifold = unit_manifold::<f64>(3);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(random_cochain::<f64>(n1, 97), vec![n1]).unwrap();
    let velocity = VelocityOneForm::new(raw, &manifold).unwrap();
    let (field, _) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();

    let snapshot = field.clone();
    let result = field.with_lift(&[]);
    assert_eq!(result, snapshot, "empty with_lift is a bit-exact no-op");
}
