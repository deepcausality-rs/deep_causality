/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::{EastCoastMetric, LorentzianMetric};
use deep_causality_physics::theories::GR;
use deep_causality_physics::theories::general_relativity::{
    AdmOps, AdmState, GrOps, flrw_metric_at, kerr_metric_at, minkowski_metric,
    schwarzschild_kretschmann, schwarzschild_metric_at,
};
use deep_causality_physics::{contract_riemann_to_lie, expand_lie_to_riemann, pair_to_lie_index};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GaugeField, Manifold, Simplex, SimplicialComplexBuilder};
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

    // Build a complex with 1 vertex (0-simplex)
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("Failed to add simplex");
    let complex = builder.build().expect("Failed to build complex");

    let num_simplices = complex.total_simplices(); // Should be 1
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex.clone(), data, 0).expect("Failed to create manifold");

    let mass = 1.0;
    let r = 10.0;

    // Use Metric Tensor as connection (GR expects g in connection slot)
    // Padded to [N, 4, 6] for GaugeField Lorentz validation
    let metric_4x4 = schwarzschild_metric_at(mass, r).unwrap();
    let m_data = metric_4x4.as_slice();

    // Construct padded data: 1 point, 4 rows, 6 cols.
    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    for p in 0..num_simplices {
        for row in 0..4 {
            for col in 0..4 {
                conn_data[p * 24 + row * 6 + col] = m_data[row * 4 + col];
            }
        }
    }
    let metric_tensor = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);

    // Riemann tensor in Lie-algebra form [N, 4, 4, 6]
    // For a proper test, we would compute this from Christoffel symbols.
    // Here we use zeros, which should give K = 0.
    let riemann_data = vec![0.0; num_simplices * 4 * 4 * 6];
    let riemann = CausalTensor::from_vec(riemann_data, &[num_simplices, 4, 4, 6]);

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();

    // Create GR field
    let gravity: GR = GaugeField::new(base, topo_metric, metric_tensor, riemann).unwrap();

    // Verify gauge group name
    assert_eq!(gravity.gauge_group_name(), "SO(3,1)");

    // Test Kretschmann scalar computation - should now succeed
    let k_res = gravity.kretschmann_scalar();
    assert!(
        k_res.is_ok(),
        "Kretschmann scalar should compute successfully with Lie→Geometric mapping"
    );

    // With zero Riemann tensor, K should be 0
    let k_value = k_res.unwrap();
    assert!(
        k_value.abs() < 1e-12,
        "Kretschmann scalar should be 0 for zero Riemann, got {}",
        k_value
    );
}

#[test]
fn test_geodesic_deviation_interface() {
    // Use N=1 for simple mocking
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("Failed to add simplex");
    let complex = builder.build().expect("Failed to build complex");

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");
    let dim = 4;

    // Mock Metric Tensor (Minkowski-like for simplicity) padded to [1, 4, 6]
    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    // East Coast signature (-1, +1, +1, +1)
    conn_data[0] = -1.0; // g_00
    conn_data[7] = 1.0; // g_11
    conn_data[14] = 1.0; // g_22
    conn_data[21] = 1.0; // g_33
    let metric_tensor = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);

    // Riemann tensor in Lie-algebra form [N, 4, 4, 6]
    // Zeros for flat spacetime - no tidal forces
    let riemann = CausalTensor::from_vec(
        vec![0.0; num_simplices * 4 * 4 * 6],
        &[num_simplices, 4, 4, 6],
    );

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gravity: GR = GaugeField::new(base, topo_metric, metric_tensor, riemann).unwrap();

    let velocity = vec![1.0, 0.0, 0.0, 0.0];
    let separation = vec![0.0, 1.0, 0.0, 0.0];

    // Geodesic deviation should succeed with Lie→Geometric expansion
    let deviation = gravity.geodesic_deviation(&velocity, &separation);
    assert!(
        deviation.is_ok(),
        "Geodesic deviation should compute successfully, got {:?}",
        deviation.err()
    );

    let result = deviation.unwrap();
    assert_eq!(result.len(), dim, "Result should have 4 components");

    // For flat spacetime (zero Riemann), deviation acceleration should be zero
    for (i, &val) in result.iter().enumerate() {
        assert!(
            val.abs() < 1e-12,
            "Deviation component {} should be 0 for flat spacetime, got {}",
            i,
            val
        );
    }
}

// ============================================================================
// Lie ↔ Geometric Mapping Tests
// ============================================================================

#[test]
fn test_pair_to_lie_index() {
    assert_eq!(pair_to_lie_index(0, 1), Some(0));
    assert_eq!(pair_to_lie_index(0, 2), Some(1));
    assert_eq!(pair_to_lie_index(0, 3), Some(2));
    assert_eq!(pair_to_lie_index(1, 2), Some(3));
    assert_eq!(pair_to_lie_index(1, 3), Some(4));
    assert_eq!(pair_to_lie_index(2, 3), Some(5));

    // Invalid cases
    assert_eq!(pair_to_lie_index(1, 1), None); // diagonal
    assert_eq!(pair_to_lie_index(2, 1), None); // wrong order
    assert_eq!(pair_to_lie_index(4, 5), None); // out of range
}

#[test]
fn test_lie_index_to_pair() {
    use deep_causality_physics::theories::general_relativity::lie_index_to_pair;

    assert_eq!(lie_index_to_pair(0), Some((0, 1)));
    assert_eq!(lie_index_to_pair(1), Some((0, 2)));
    assert_eq!(lie_index_to_pair(2), Some((0, 3)));
    assert_eq!(lie_index_to_pair(3), Some((1, 2)));
    assert_eq!(lie_index_to_pair(4), Some((1, 3)));
    assert_eq!(lie_index_to_pair(5), Some((2, 3)));
    assert_eq!(lie_index_to_pair(6), None);
}

#[test]
fn test_roundtrip_lie_geometric() {
    // Create a sample Lie-algebra tensor [4, 4, 6]
    let mut lie_data = vec![0.0; 4 * 4 * 6];
    // Set some non-zero values
    lie_data[6] = 1.0; // R^0_1 at (μ,ν)=(0,1)
    lie_data[2 * 4 * 6 + 3 * 6 + 5] = 2.5; // R^2_3 at (μ,ν)=(2,3)

    let lie_tensor = CausalTensor::from_vec(lie_data.clone(), &[4, 4, 6]);

    // Expand to geometric
    let riemann = expand_lie_to_riemann(&lie_tensor).unwrap();
    assert_eq!(riemann.shape(), &[4, 4, 4, 4]);

    // Contract back to Lie
    let lie_back = contract_riemann_to_lie(&riemann).unwrap();
    assert_eq!(lie_back.shape(), &[4, 4, 6]);

    // Verify roundtrip
    let original = lie_tensor.as_slice();
    let recovered = lie_back.as_slice();
    for i in 0..original.len() {
        assert!(
            (original[i] - recovered[i]).abs() < 1e-12,
            "Mismatch at index {}: {} vs {}",
            i,
            original[i],
            recovered[i]
        );
    }
}

#[test]
fn test_antisymmetry_preserved() {
    // Create Lie tensor with known value
    let mut lie_data = vec![0.0; 4 * 4 * 6];
    lie_data[0] = PI; // R^0_0 at (μ,ν)=(0,1) → lie_idx=0

    let lie_tensor = CausalTensor::from_vec(lie_data, &[4, 4, 6]);
    let riemann = expand_lie_to_riemann(&lie_tensor).unwrap();
    let r_data = riemann.as_slice();

    // R^0_0_01 should be PI
    let r_0_0_01 = r_data[1]; // [0,0,0,1]
    assert!((r_0_0_01 - PI).abs() < 1e-12);

    // R^0_0_10 should be -PI (antisymmetry)
    let r_0_0_10 = r_data[4]; // [0,0,1,0]
    assert!((r_0_0_10 + PI).abs() < 1e-12);

    // Diagonal should be zero
    let r_0_0_00 = r_data[0]; // [0,0,0,0]
    assert!(r_0_0_00.abs() < 1e-12);
}

// ============================================================================
// Tests for Three Remaining GR Issues
// ============================================================================

#[test]
fn test_adm_with_christoffel() {
    // Test that ADM momentum constraint works when Christoffel symbols are provided
    let gamma = CausalTensor::identity(&[3, 3]).unwrap();
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    // Zero Christoffel symbols (flat space)
    let christoffel = CausalTensor::zeros(&[3, 3, 3]);

    let state = AdmState::with_christoffel(
        gamma,
        k,
        alpha,
        beta,
        0.0, // R = 0 for flat
        christoffel,
    );

    // Momentum constraint should now return Ok, not Err
    let m = state.momentum_constraint(None);
    assert!(
        m.is_ok(),
        "Momentum constraint should succeed with Christoffel symbols: {:?}",
        m.err()
    );

    // For flat space with K=0, momentum constraint should be zero
    let m_vec = m.unwrap();
    assert_eq!(m_vec.shape(), &[3]);
    for (i, &val) in m_vec.as_slice().iter().enumerate() {
        assert!(
            val.abs() < 1e-12,
            "M_{} should be 0 for flat space with K=0, got {}",
            i,
            val
        );
    }
}

#[test]
fn test_multipoint_expand_lie_to_riemann() {
    use deep_causality_physics::theories::general_relativity::expand_lie_to_riemann;

    // Create a 3-point Lie tensor [3, 4, 4, 6]
    let num_points = 3;
    let elem_per_point = 4 * 4 * 6;
    let mut lie_data = vec![0.0; num_points * elem_per_point];

    // Set different values for each point at the same Lie index
    for p in 0..num_points {
        // R^0_0 at (μ,ν)=(0,1) → lie_idx=0
        lie_data[p * elem_per_point] = (p + 1) as f64;
    }

    let lie_tensor = CausalTensor::from_vec(lie_data, &[num_points, 4, 4, 6]);

    // Expand to geometric Riemann
    let riemann = expand_lie_to_riemann(&lie_tensor).unwrap();

    // Should be [3, 4, 4, 4, 4]
    assert_eq!(riemann.shape(), &[num_points, 4, 4, 4, 4]);

    // Verify each point has its correct value
    let r_data = riemann.as_slice();
    let elem_per_riemann = 256;
    for p in 0..num_points {
        // R^0_0_01 should be (p+1)
        let r_0_0_01 = r_data[p * elem_per_riemann + 1];
        assert!(
            (r_0_0_01 - (p + 1) as f64).abs() < 1e-12,
            "Point {} R^0_0_01 should be {}, got {}",
            p,
            p + 1,
            r_0_0_01
        );

        // R^0_0_10 should be -(p+1) (antisymmetry)
        let r_0_0_10 = r_data[p * elem_per_riemann + 4];
        assert!(
            (r_0_0_10 + (p + 1) as f64).abs() < 1e-12,
            "Point {} R^0_0_10 should be -{}, got {}",
            p,
            p + 1,
            r_0_0_10
        );
    }
}

#[test]
fn test_compute_riemann_from_christoffel() {
    use deep_causality_topology::{Simplex, SimplicialComplexBuilder};

    // Build a simple complex
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("Failed to add simplex");
    let complex = builder.build().expect("Failed to build complex");

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");

    // Create connection tensor (simulating Christoffel symbol data) [N, 4, 6]
    let conn_data = vec![0.0; num_simplices * 4 * 6];
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);

    // Create field strength (will be replaced by computation)
    let fs_data = vec![0.0; num_simplices * 4 * 4 * 6];
    let field_strength = CausalTensor::from_vec(fs_data, &[num_simplices, 4, 4, 6]);

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();

    // Create GR field with Christoffel in connection slot
    let gr: GR = GaugeField::new(base, topo_metric, connection, field_strength).unwrap();

    // Test that compute_riemann_from_christoffel returns the expected shape
    let riemann = gr.compute_riemann_from_christoffel();

    // Should have shape [N, 4, 4, 6]
    assert_eq!(riemann.shape()[0], num_simplices);
    assert_eq!(riemann.shape()[1], 4);
    assert_eq!(riemann.shape()[2], 4);
    assert_eq!(riemann.shape()[3], 6);
}

#[test]
fn test_momentum_constraint_field() {
    use deep_causality_topology::{Simplex, SimplicialComplexBuilder};

    // Build a simple complex with 3 vertices (3 points for multi-point test)
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("add simplex");
    builder
        .add_simplex(Simplex::new(vec![1]))
        .expect("add simplex");
    builder
        .add_simplex(Simplex::new(vec![2]))
        .expect("add simplex");
    let complex = builder.build().expect("build complex");

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).expect("create manifold");

    // Create 4x4 Minkowski metric: diag(-1, 1, 1, 1) for each point
    // This provides a valid non-singular spatial 3-metric γ_ij = diag(1, 1, 1)
    let mut metric_4x4 = vec![0.0; num_simplices * 16];
    for p in 0..num_simplices {
        let base_idx = p * 16;
        metric_4x4[base_idx] = -1.0; // g_00 = -1
        metric_4x4[base_idx + 5] = 1.0; // g_11 = 1 (γ_00)
        metric_4x4[base_idx + 10] = 1.0; // g_22 = 1 (γ_11)
        metric_4x4[base_idx + 15] = 1.0; // g_33 = 1 (γ_22)
    }
    let connection = CausalTensor::from_vec(metric_4x4, &[num_simplices, 4, 4]);

    let fs_data = vec![0.0; num_simplices * 4 * 4 * 6];
    let field_strength = CausalTensor::from_vec(fs_data, &[num_simplices, 4, 4, 6]);

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR = GaugeField::new(base, topo_metric, connection, field_strength).unwrap();

    // Create extrinsic curvature K_ij = 0 (flat space)
    let k_data = vec![0.0; num_simplices * 3 * 3];
    let k_tensor = CausalTensor::from_vec(k_data, &[num_simplices, 3, 3]);

    // Test momentum_constraint_field
    let result = gr.momentum_constraint_field(&k_tensor, None);
    assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());

    let m = result.unwrap();

    // Should have shape [N, 3]
    assert_eq!(m.shape()[0], num_simplices);
    assert_eq!(m.shape()[1], 3);

    // For flat space with K=0, momentum constraint should be zero
    for val in m.as_slice() {
        assert!(
            val.abs() < 1e-12,
            "Expected M_i = 0 for flat space with K=0, got {}",
            val
        );
    }
}
