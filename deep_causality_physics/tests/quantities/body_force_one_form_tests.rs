/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constructor and rejection-branch tests for `BodyForceOneForm` (grade 1).

use deep_causality_physics::BodyForceOneForm;
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
    let n1 = manifold.complex().num_cells(1);
    let g = BodyForceOneForm::new(
        CausalTensor::new(vec![-9.81; n1], vec![n1]).unwrap(),
        &manifold,
    )
    .unwrap();
    assert_eq!(g.len(), n1);
    assert!(!g.is_empty());
    assert_eq!(g.as_tensor().as_slice()[0], -9.81);
}

#[test]
fn rejects_length_mismatch() {
    let manifold = unit_manifold(3);
    let n0 = manifold.complex().num_cells(0);
    let bad = CausalTensor::new(vec![0.0; n0], vec![n0]).unwrap();
    let err = BodyForceOneForm::new(bad, &manifold).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("BodyForceOneForm") && msg.contains("grade-1"));
}

#[test]
fn rejects_nan_coefficient() {
    let manifold = unit_manifold(3);
    let n1 = manifold.complex().num_cells(1);
    let mut data = vec![0.0; n1];
    data[5] = f64::NAN;
    let err =
        BodyForceOneForm::new(CausalTensor::new(data, vec![n1]).unwrap(), &manifold).unwrap_err();
    assert!(format!("{err}").contains("non-finite coefficient at index 5"));
}

#[test]
fn rejects_positive_infinity_coefficient() {
    let manifold = unit_manifold(3);
    let n1 = manifold.complex().num_cells(1);
    let mut data = vec![0.0; n1];
    data[0] = f64::INFINITY;
    let err =
        BodyForceOneForm::new(CausalTensor::new(data, vec![n1]).unwrap(), &manifold).unwrap_err();
    assert!(format!("{err}").contains("Numerical Instability"));
}

#[test]
fn rejects_negative_infinity_coefficient() {
    let manifold = unit_manifold(3);
    let n1 = manifold.complex().num_cells(1);
    let mut data = vec![0.0; n1];
    data[n1 - 1] = f64::NEG_INFINITY;
    let err =
        BodyForceOneForm::new(CausalTensor::new(data, vec![n1]).unwrap(), &manifold).unwrap_err();
    assert!(format!("{err}").contains("Numerical Instability"));
}

#[test]
fn derives_debug_clone_partial_eq() {
    let manifold = unit_manifold(3);
    let n1 = manifold.complex().num_cells(1);
    let g = BodyForceOneForm::new(
        CausalTensor::new(vec![1.0; n1], vec![n1]).unwrap(),
        &manifold,
    )
    .unwrap();
    let h = g.clone();
    assert_eq!(g, h);
    assert!(format!("{g:?}").contains("BodyForceOneForm"));
}
