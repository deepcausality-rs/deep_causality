/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Density, PhysicalField, alfven_speed_kernel, ideal_induction_kernel, magnetic_pressure_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

fn create_dummy_manifold() -> Manifold<f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num = complex.total_simplices();
    Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; num], vec![num]).unwrap(),
        0,
    )
    .unwrap()
}

#[test]
fn test_alfven_speed() {
    let b_vec = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b_field = PhysicalField::new(b_vec);
    let rho = Density::new(1.0).unwrap();
    let mu0 = 1.0;

    let res = alfven_speed_kernel(&b_field, &rho, mu0);
    assert!(res.is_ok());
    // vA = |B| / sqrt(mu0 * rho) = 1 / 1 = 1
    assert!((res.unwrap().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_alfven_speed_errors() {
    let b_vec = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b_field = PhysicalField::new(b_vec);
    let rho_valid = Density::new(1.0).unwrap();

    // Permeability error
    assert!(alfven_speed_kernel(&b_field, &rho_valid, 0.0).is_err());
    assert!(alfven_speed_kernel(&b_field, &rho_valid, -1.0).is_err());

    // Density error (zero)
    let rho_zero = Density::new_unchecked(0.0);
    assert!(alfven_speed_kernel(&b_field, &rho_zero, 1.0).is_err());
}

#[test]
fn test_magnetic_pressure() {
    let b_vec = CausalMultiVector::new(vec![0.0, 2.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b_field = PhysicalField::new(b_vec);
    let mu0 = 1.0;

    let res = magnetic_pressure_kernel(&b_field, mu0);
    assert!(res.is_ok());
    // P = B^2 / 2mu0 = 4 / 2 = 2
    assert!((res.unwrap().value() - 2.0).abs() < 1e-10);
}

#[test]
fn test_magnetic_pressure_error() {
    let b_vec = CausalMultiVector::new(vec![0.0, 2.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b_field = PhysicalField::new(b_vec);
    assert!(magnetic_pressure_kernel(&b_field, 0.0).is_err());
}

#[test]
fn test_ideal_induction() {
    let m = create_dummy_manifold();
    let res = ideal_induction_kernel(&m, &m);
    assert!(
        res.is_ok(),
        "Ideal induction kernel failed: {:?}",
        res.err()
    );
    let tensor = res.unwrap();
    assert!(!tensor.is_empty());
    // For zero inputs (create_dummy_manifold inits with 0), output should be 0.
    for val in tensor.as_slice() {
        assert_eq!(*val, 0.0);
    }
}

#[test]
fn test_ideal_induction_dimension_error() {
    // Manifold with only 0 and 1 skeletons (1D manifold/graph)
    // Points for a single line segment
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0], vec![2, 2]).unwrap();
    let point_cloud = PointCloud::new(
        points,
        CausalTensor::new(vec![0.0, 0.0], vec![2]).unwrap(),
        0,
    )
    .unwrap();
    let complex = point_cloud.triangulate(1.5).unwrap();
    let num = complex.total_simplices();
    let m = Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; num], vec![num]).unwrap(),
        0,
    )
    .unwrap();

    let res = ideal_induction_kernel(&m, &m);
    assert!(res.is_err());
}
