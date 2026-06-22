/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constructor and rejection-branch tests for `VorticityTwoForm` (grade 2).

use deep_causality_physics::VorticityTwoForm;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

fn unit_manifold(n: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

#[test]
fn constructs_with_valid_field_and_exposes_getters() {
    let manifold = unit_manifold(3);
    let n2 = manifold.complex().num_cells(2);
    let w = VorticityTwoForm::new(
        CausalTensor::new(vec![-0.25; n2], vec![n2]).unwrap(),
        &manifold,
    )
    .unwrap();
    assert_eq!(w.len(), n2);
    assert!(!w.is_empty());
    assert_eq!(w.as_tensor().as_slice()[0], -0.25);
}

#[test]
fn rejects_length_mismatch() {
    let manifold = unit_manifold(3);
    // Grade-1 length supplied where grade-2 expected.
    let n1 = manifold.complex().num_cells(1);
    let bad = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let err = VorticityTwoForm::new(bad, &manifold).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("VorticityTwoForm") && msg.contains("grade-2"));
}

#[test]
fn rejects_nan_coefficient() {
    let manifold = unit_manifold(3);
    let n2 = manifold.complex().num_cells(2);
    let mut data = vec![0.0; n2];
    data[3] = f64::NAN;
    let err =
        VorticityTwoForm::new(CausalTensor::new(data, vec![n2]).unwrap(), &manifold).unwrap_err();
    assert!(format!("{err}").contains("non-finite coefficient at index 3"));
}

#[test]
fn rejects_positive_infinity_coefficient() {
    let manifold = unit_manifold(3);
    let n2 = manifold.complex().num_cells(2);
    let mut data = vec![0.0; n2];
    data[0] = f64::INFINITY;
    let err =
        VorticityTwoForm::new(CausalTensor::new(data, vec![n2]).unwrap(), &manifold).unwrap_err();
    assert!(format!("{err}").contains("Numerical Instability"));
}

#[test]
fn rejects_negative_infinity_coefficient() {
    let manifold = unit_manifold(3);
    let n2 = manifold.complex().num_cells(2);
    let mut data = vec![0.0; n2];
    data[n2 - 1] = f64::NEG_INFINITY;
    let err =
        VorticityTwoForm::new(CausalTensor::new(data, vec![n2]).unwrap(), &manifold).unwrap_err();
    assert!(format!("{err}").contains("Numerical Instability"));
}

#[test]
fn derives_debug_clone_partial_eq() {
    let manifold = unit_manifold(3);
    let n2 = manifold.complex().num_cells(2);
    let w = VorticityTwoForm::new(
        CausalTensor::new(vec![1.0; n2], vec![n2]).unwrap(),
        &manifold,
    )
    .unwrap();
    let x = w.clone();
    assert_eq!(w, x);
    assert!(format!("{w:?}").contains("VorticityTwoForm"));
}
