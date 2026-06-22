/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::utils_tests::{random_cochain, unit_manifold};
use deep_causality_physics::{SolenoidalField, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::ChainComplex;
use deep_causality_topology::HodgeDecomposition;

#[test]
fn from_leray_projection_propagates_projection_failure() {
    // Velocity validated against a 3×3 lattice, projected against a 4×4 one:
    // the inner leray_project dimension mismatch surfaces as a typed
    // PhysicsError::TopologyError.
    let m3 = unit_manifold::<f64>(3);
    let m4 = unit_manifold::<f64>(4);
    let n1_3 = m3.complex().num_cells(1);
    let velocity =
        VelocityOneForm::new(CausalTensor::new(vec![1.0; n1_3], vec![n1_3]).unwrap(), &m3).unwrap();

    let err = SolenoidalField::from_leray_projection(&velocity, &m4).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("Topology Error") && msg.contains("Leray projection failed"),
        "got: {msg}"
    );
}

#[test]
fn from_hodge_projection_rejects_wrong_grade() {
    let manifold = unit_manifold::<f64>(3);
    let n0 = manifold.complex().num_cells(0);
    let scalar_field = CausalTensor::new(random_cochain::<f64>(n0, 71), vec![n0]).unwrap();
    let decomposition = manifold.hodge_decompose(&scalar_field, 0).unwrap();

    let err = SolenoidalField::from_hodge_projection(&decomposition).unwrap_err();
    assert!(format!("{err}").contains("grade-1 decomposition"));
}

#[test]
fn from_hodge_projection_rejects_component_length_mismatch() {
    // Adversarial decomposition with disagreeing component lengths.
    let exact = CausalTensor::new(vec![0.0_f64; 4], vec![4]).unwrap();
    let co_exact = CausalTensor::new(vec![0.0_f64; 4], vec![4]).unwrap();
    let harmonic = CausalTensor::new(vec![0.0_f64; 3], vec![3]).unwrap();
    let decomposition = HodgeDecomposition::new(exact, co_exact, harmonic, 1);

    let err = SolenoidalField::from_hodge_projection(&decomposition).unwrap_err();
    assert!(format!("{err}").contains("component length mismatch"));
}

#[test]
fn from_hodge_projection_rejects_non_finite_components() {
    let exact = CausalTensor::new(vec![0.0_f64; 4], vec![4]).unwrap();
    let co_exact = CausalTensor::new(vec![0.0, f64::NAN, 0.0, 0.0], vec![4]).unwrap();
    let harmonic = CausalTensor::new(vec![0.0_f64; 4], vec![4]).unwrap();
    let decomposition = HodgeDecomposition::new(exact, co_exact, harmonic, 1);

    let err = SolenoidalField::from_hodge_projection(&decomposition).unwrap_err();
    assert!(format!("{err}").contains("non-finite coefficient at index 1"));
}
