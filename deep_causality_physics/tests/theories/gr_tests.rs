/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::theories::GR;
use deep_causality_physics::theories::gr::{
    AdmOps, AdmState, GrOps, flrw_metric_at, kerr_metric_at, minkowski_metric,
    schwarzschild_christoffel_at, schwarzschild_kretschmann, schwarzschild_metric_at,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::GaugeField;
use deep_causality_topology::Manifold;
use std::f64::consts::PI;

#[test]
fn test_minkowski_spacetime() {
    let metric = minkowski_metric();
    assert_eq!(metric.shape(), &[4, 4]);

    // Verify signature (-+++)
    let diag = metric.as_slice();
    assert_eq!(diag[0], -1.0); // g_00
    assert_eq!(diag[5], 1.0); // g_11
    assert_eq!(diag[10], 1.0); // g_22
    assert_eq!(diag[15], 1.0); // g_33
}

#[test]
fn test_schwarzschild_metric_properties() {
    let mass = 1.0;
    let r = 10.0;
    let metric = schwarzschild_metric_at(mass, r).expect("Should create metric");

    let data = metric.as_slice();
    // g_tt = -(1 - 2M/r) = -(1 - 2/10) = -0.8
    assert!((data[0] - (-0.8)).abs() < 1e-10);

    // g_rr = 1/(1 - 2M/r) = 1/0.8 = 1.25
    assert!((data[5] - 1.25).abs() < 1e-10);

    // g_theta = r^2 = 100
    assert!((data[10] - 100.0).abs() < 1e-10);
}

#[test]
fn test_schwarzschild_curvature_invariants() {
    let mass = 1.0;
    let r = 10.0;

    // Direct calculation of K
    let k_exact = schwarzschild_kretschmann(mass, r);
    // K = 48 M^2 / r^6 = 48 * 1 / 10^6 = 0.000048
    assert!((k_exact - 48.0e-6).abs() < 1e-12);
}

#[test]
fn test_kerr_black_hole() {
    let mass = 2.0;
    let r = 20.0;
    let theta = PI / 2.0; // Equatorial

    // Case 1: Non-rotating (a=0) limit -> Should match Schwarzschild
    let kerr0 = kerr_metric_at(mass, 0.0, r, theta).unwrap();
    let schw = schwarzschild_metric_at(mass, r).unwrap();

    let k_data = kerr0.as_slice();
    let s_data = schw.as_slice();

    for i in 0..16 {
        assert!(
            (k_data[i] - s_data[i]).abs() < 1e-10,
            "Mismatch at index {}",
            i
        );
    }

    // Case 2: Extreme Kerr (a=M)
    // At horizon r = M + sqrt(M^2 - a^2) = M
    // Here we test outside horizon
    let _kerr_rot = kerr_metric_at(mass, mass * 0.9, r, theta).unwrap();
}

#[test]
fn test_flrw_cosmology() {
    let a = 2.0; // Scale factor
    let r = 5.0;
    let theta = PI / 2.0;

    // Flat universe (k=0)
    let flrw = flrw_metric_at(a, 0.0, r, theta).unwrap();
    let data = flrw.as_slice();

    // ds^2 = -dt^2 + a^2(dr^2 + r^2...)
    assert_eq!(data[0], -1.0); // g_tt
    assert_eq!(data[5], a * a); // g_rr for flat k=0
}

#[test]
fn test_adm_structures() {
    let gamma = CausalTensor::identity(&[3, 3]).unwrap();
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    let state = AdmState::new(gamma.clone(), k, alpha.clone(), beta.clone(), 0.0);

    // Test hamiltonian constraint interface
    let h = state.hamiltonian_constraint(None).unwrap();
    assert_eq!(h.shape(), &[1]);
    // Expect 0 for flat slice with K=0
    assert_eq!(h.as_slice()[0], 0.0);

    // Test Case 2: Non-zero expansion (K = I)
    // Tr K = 3, K_ij K^ij = 3 => H = 3^2 - 3 = 6
    let k_expanding = CausalTensor::identity(&[3, 3]).unwrap();
    let state_expanding =
        AdmState::new(gamma.clone(), k_expanding, alpha.clone(), beta.clone(), 0.0);
    let h_expanding = state_expanding.hamiltonian_constraint(None).unwrap();
    assert!(
        (h_expanding.as_slice()[0] - 6.0).abs() < 1e-10,
        "H should be 6 for isotropic expansion with R=0"
    );

    // Test momentum constraint interface (Expect Error due to missing derivatives)
    assert!(
        state.momentum_constraint(None).is_err(),
        "Momentum constraint should error without spatial derivatives"
    );
}

#[test]
fn test_gr_gauge_field_integration() {
    // Construct a GR field for Schwarzschild
    use deep_causality_topology::{Simplex, SimplicialComplexBuilder};

    // Build a complex with at least one 3-simplex (tetrahedron) for 4D manifold basic structure
    let mut builder = SimplicialComplexBuilder::new(3);
    builder
        .add_simplex(Simplex::new(vec![0, 1, 2, 3]))
        .expect("Failed to add simplex");
    let complex = builder.build().expect("Failed to build complex");

    let num_simplices = complex.total_simplices();
    // Data tensor must match the number of simplices
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex.clone(), data, 0).expect("Failed to create manifold");

    let mass = 1.0;
    let r = 10.0;

    // Use Christoffel as connection
    let christoffel = schwarzschild_christoffel_at(mass, r).unwrap();

    // Use Riemann as field strength (placeholder zero for test setup simplification)
    let riemann_data = vec![0.0; 4 * 4 * 4 * 4];
    let riemann = deep_causality_tensor::CausalTensor::from_vec(riemann_data, &[4, 4, 4, 4]);

    // Create GR field
    let gravity: GR = GaugeField::with_default_metric(base, christoffel, riemann).unwrap();

    // Verify it implements GrOps
    assert_eq!(gravity.gauge_group_name(), "SO(3,1)");

    // Test default GrOps methods (sanity checks)
    let k = gravity.kretschmann_scalar().unwrap();
    assert_eq!(k, 0.0); // Using zero Riemann for this test case
}

#[test]
fn test_geodesic_deviation_interface() {
    use deep_causality_topology::{Simplex, SimplicialComplexBuilder};

    let mut builder = SimplicialComplexBuilder::new(3);
    builder
        .add_simplex(Simplex::new(vec![0, 1, 2, 3]))
        .expect("Failed to add simplex");
    let complex = builder.build().expect("Failed to build complex");

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");
    let dim = 4;

    // Mock data - needs to match constructor expectations shape-wise
    let christoffel = deep_causality_tensor::CausalTensor::from_vec(vec![0.0; 64], &[4, 4, 4]);
    let riemann = deep_causality_tensor::CausalTensor::from_vec(vec![1.0; 256], &[4, 4, 4, 4]);

    let gravity: GR = GaugeField::with_default_metric(base, christoffel, riemann).unwrap();

    let velocity = vec![1.0, 0.0, 0.0, 0.0];
    let separation = vec![0.0, 1.0, 0.0, 0.0];

    // Should run without error
    let deviation = gravity.geodesic_deviation(&velocity, &separation);
    assert!(deviation.is_ok());
    assert_eq!(deviation.unwrap().len(), dim);
}
