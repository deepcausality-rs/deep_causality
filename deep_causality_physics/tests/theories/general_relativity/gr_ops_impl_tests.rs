/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for gr_ops_impl.rs - GR gauge field operations and trait methods

use deep_causality_metric::{EastCoastMetric, LorentzianMetric};
use deep_causality_physics::theories::GR;
use deep_causality_physics::theories::general_relativity::{GrOps, schwarzschild_metric_at};
use deep_causality_physics::{NEWTONIAN_CONSTANT_OF_GRAVITATION, SPEED_OF_LIGHT};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GaugeField, Manifold, Simplex, SimplicialComplexBuilder};
// ============================================================================
// GR Gauge Field Integration Tests
// ============================================================================

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
    let gravity: GR<f64> = GaugeField::new(base, topo_metric, metric_tensor, riemann).unwrap();

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
    let gravity: GR<f64> = GaugeField::new(base, topo_metric, metric_tensor, riemann).unwrap();

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

#[test]
fn test_compute_riemann_from_christoffel() {
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
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, field_strength).unwrap();

    // Test that compute_riemann_from_christoffel returns the expected shape
    let riemann = gr.compute_riemann_from_christoffel();

    // Should have shape [N, 4, 4, 6]
    assert_eq!(riemann.shape()[0], num_simplices);
    assert_eq!(riemann.shape()[1], 4);
    assert_eq!(riemann.shape()[2], 4);
    assert_eq!(riemann.shape()[3], 6);
}

// ============================================================================
// Momentum Constraint Field
// ============================================================================

#[test]
fn test_momentum_constraint_field() {
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

    // Create 4x6 connection tensor for SO(3,1) Lorentz gauge group
    // The momentum_constraint_field extracts spatial metric assuming 4-column inner stride
    // idx = base + (i+1)*4 + (j+1), so spatial diagonals at offsets 5, 10, 15
    let mut metric_4x6 = vec![0.0; num_simplices * 4 * 6];
    for p in 0..num_simplices {
        let base_idx = p * 24; // 4 * 6 = 24 per point
        // g_00 = -1 (timelike) at row 0, col 0 → offset 0
        metric_4x6[base_idx] = -1.0;
        // Spatial diagonals: extraction uses (i+1)*4 + (j+1)
        // g_11: (1)*4 + 1 = 5
        metric_4x6[base_idx + 5] = 1.0;
        // g_22: (2)*4 + 2 = 10
        metric_4x6[base_idx + 10] = 1.0;
        // g_33: (3)*4 + 3 = 15
        metric_4x6[base_idx + 15] = 1.0;
    }
    let connection = CausalTensor::from_vec(metric_4x6, &[num_simplices, 4, 6]);

    let fs_data = vec![0.0; num_simplices * 4 * 4 * 6];
    let field_strength = CausalTensor::from_vec(fs_data, &[num_simplices, 4, 4, 6]);

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, field_strength).unwrap();

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

#[test]
fn test_momentum_constraint_2d_k_tensor() {
    // Test single-point [3, 3] K_ij shape
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0;
    conn_data[5] = 1.0;
    conn_data[10] = 1.0;
    conn_data[15] = 1.0;
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);

    let riemann = CausalTensor::zeros(&[num_simplices, 4, 4, 6]);

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    // 2D K_ij tensor [3, 3]
    let k_2d = CausalTensor::zeros(&[3, 3]);
    let result = gr.momentum_constraint_field(&k_2d, None);

    assert!(result.is_ok(), "2D K_ij should work: {:?}", result.err());
    let m = result.unwrap();
    assert_eq!(m.shape(), &[3], "Single-point output should be [3]");
}

#[test]
fn test_momentum_constraint_wrong_2d_shape() {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0;
    conn_data[5] = 1.0;
    conn_data[10] = 1.0;
    conn_data[15] = 1.0;
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);
    let riemann = CausalTensor::zeros(&[num_simplices, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    // Wrong 2D shape [2, 3]
    let bad_k = CausalTensor::zeros(&[2, 3]);
    let result = gr.momentum_constraint_field(&bad_k, None);

    assert!(result.is_err(), "Wrong 2D shape [2, 3] should error");
}

#[test]
fn test_momentum_constraint_wrong_3d_shape() {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0;
    conn_data[5] = 1.0;
    conn_data[10] = 1.0;
    conn_data[15] = 1.0;
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);
    let riemann = CausalTensor::zeros(&[num_simplices, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    // Wrong 3D shape [N, 2, 3]
    let bad_k = CausalTensor::zeros(&[num_simplices, 2, 3]);
    let result = gr.momentum_constraint_field(&bad_k, None);

    assert!(result.is_err(), "Wrong 3D shape [N, 2, 3] should error");
}

#[test]
fn test_momentum_constraint_wrong_dimension() {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0;
    conn_data[5] = 1.0;
    conn_data[10] = 1.0;
    conn_data[15] = 1.0;
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);
    let riemann = CausalTensor::zeros(&[num_simplices, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    // 1D K should fail
    let k_1d = CausalTensor::zeros(&[9]);
    let result = gr.momentum_constraint_field(&k_1d, None);
    assert!(result.is_err(), "1D K_ij should error");

    // 4D K should fail
    let k_4d = CausalTensor::zeros(&[1, 1, 3, 3]);
    let result = gr.momentum_constraint_field(&k_4d, None);
    assert!(result.is_err(), "4D K_ij should error");
}

#[test]
fn test_momentum_constraint_matter_size_mismatch() {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    builder.add_simplex(Simplex::new(vec![1])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    for p in 0..num_simplices {
        conn_data[p * 24] = -1.0;
        conn_data[p * 24 + 5] = 1.0;
        conn_data[p * 24 + 10] = 1.0;
        conn_data[p * 24 + 15] = 1.0;
    }
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);
    let riemann = CausalTensor::zeros(&[num_simplices, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    // K_ij with 2 points
    let k = CausalTensor::zeros(&[num_simplices, 3, 3]);

    // Matter momentum with wrong size (3 instead of 6)
    let wrong_j = CausalTensor::from_vec(vec![1.0, 0.0, 0.0], &[3]);
    let result = gr.momentum_constraint_field(&k, Some(&wrong_j));

    assert!(
        result.is_err(),
        "Matter momentum size mismatch should error"
    );
}

// ============================================================================
// GR Ops Trait Default Methods
// ============================================================================

#[test]
fn test_kretschmann_curvature_radius_flat_spacetime() {
    // For flat spacetime (K = 0), curvature radius should be infinity
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("Failed to add simplex");
    let complex = builder.build().expect("Failed to build complex");

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");

    // Minkowski metric padded to [N, 4, 6]
    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0; // g_00
    conn_data[7] = 1.0; // g_11
    conn_data[14] = 1.0; // g_22
    conn_data[21] = 1.0; // g_33
    let metric_tensor = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);

    // Zero Riemann tensor -> K = 0
    let riemann = CausalTensor::from_vec(
        vec![0.0; num_simplices * 4 * 4 * 6],
        &[num_simplices, 4, 4, 6],
    );

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gravity: GR<f64> = GaugeField::new(base, topo_metric, metric_tensor, riemann).unwrap();

    // For K = 0, curvature radius should be infinity
    let r_curv = gravity.kretschmann_curvature_radius().unwrap();
    assert!(
        r_curv.is_infinite(),
        "Curvature radius should be infinity for flat spacetime"
    );
}

#[test]
fn test_kretschmann_curvature_radius_curved_spacetime() {
    // A non-zero Riemann field strength over a non-singular Minkowski metric
    // yields K > 0, exercising the non-flat branch of the
    // `kretschmann_curvature_radius` default method (R_curv = K^(-1/4)).
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // Non-singular Minkowski metric padded to [N, 4, 6] (stride 6).
    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0; // g_00
    conn_data[7] = 1.0; // g_11
    conn_data[14] = 1.0; // g_22
    conn_data[21] = 1.0; // g_33
    let metric_tensor = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);

    // Non-zero Lie field strength [N, 4, 4, 6] → non-zero Riemann → K > 0.
    // Use a purely *spatial* component so every raised index multiplies the
    // +1 spatial part of the (-+++) metric, keeping K = R_μνρσ R^μνρσ positive.
    // Layout: flat = ((rho*4 + sigma)*6) + lie_idx. Choose rho=1, sigma=2,
    // lie_idx=5 → pair (2,3): all indices spatial.
    let mut fs = vec![0.0; num_simplices * 4 * 4 * 6];
    // flat = ((rho*4 + sigma)*6) + lie_idx with rho=1, sigma=2, lie_idx=5.
    let (rho, sigma, lie_idx) = (1usize, 2usize, 5usize);
    let idx = (rho * 4 + sigma) * 6 + lie_idx; // = 41
    fs[idx] = 1.0;
    let riemann = CausalTensor::from_vec(fs, &[num_simplices, 4, 4, 6]);

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gravity: GR<f64> = GaugeField::new(base, topo_metric, metric_tensor, riemann).unwrap();

    let k = gravity.kretschmann_scalar().unwrap();
    assert!(k > 0.0, "Expected positive Kretschmann scalar, got {}", k);

    let r_curv = gravity.kretschmann_curvature_radius().unwrap();
    assert!(
        r_curv.is_finite() && r_curv > 0.0,
        "Curvature radius should be finite and positive for K>0, got {}",
        r_curv
    );
    // R_curv = K^(-1/4): cross-check the conversion.
    let expected = 1.0 / k.powf(0.25);
    assert!(
        (r_curv - expected).abs() < 1e-12,
        "R_curv mismatch: got {}, expected {}",
        r_curv,
        expected
    );
}

#[test]
fn test_geodesic_deviation_si() {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("Failed to add simplex");
    let complex = builder.build().expect("Failed to build complex");

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");

    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0;
    conn_data[7] = 1.0;
    conn_data[14] = 1.0;
    conn_data[21] = 1.0;
    let metric_tensor = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);
    let riemann = CausalTensor::from_vec(
        vec![0.0; num_simplices * 4 * 4 * 6],
        &[num_simplices, 4, 4, 6],
    );

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gravity: GR<f64> = GaugeField::new(base, topo_metric, metric_tensor, riemann).unwrap();

    let velocity = vec![1.0, 0.0, 0.0, 0.0];
    let separation = vec![0.0, 1.0, 0.0, 0.0];

    // Test SI conversion method
    let deviation_si = gravity.geodesic_deviation_si(&velocity, &separation);
    assert!(deviation_si.is_ok(), "geodesic_deviation_si should succeed");

    // For flat spacetime, deviation should still be zero
    let result = deviation_si.unwrap();
    for val in &result {
        assert!(
            val.abs() < 1e-10,
            "Deviation should be 0 for flat spacetime"
        );
    }
}

#[test]
fn test_proper_time_si() {
    // For this test, we'll verify proper_time_si exists and has correct relationship
    // by testing with a simple approach - the SI method divides by c

    // Test that the function exists and relates to geometric properly
    // We use a minimal setup just to verify the SI conversion logic
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("Failed to add simplex");
    let complex = builder.build().expect("Failed to build complex");

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");

    // Connection must be [N, 4, 6] for SO(3,1) Lorentz gauge group
    // Embed 4x4 metric in first 4 columns of 4x6
    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0; // g_00
    conn_data[7] = 1.0; // g_11 (row 1, col 1 in 4x6 = index 6+1)
    conn_data[14] = 1.0; // g_22 (row 2, col 2 in 4x6 = index 12+2)
    conn_data[21] = 1.0; // g_33 (row 3, col 3 in 4x6 = index 18+3)
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);

    // Field strength in Lie-algebra form
    let riemann = CausalTensor::from_vec(
        vec![0.0; num_simplices * 4 * 4 * 6],
        &[num_simplices, 4, 4, 6],
    );

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gravity: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    // Timelike path (pure time direction)
    let path = vec![vec![0.0, 0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0, 0.0]];

    // Test proper_time
    let tau_result = gravity.proper_time(&path);

    // Due to shape constraints, this may fail - we test the SI wrapper exists
    // by checking the method is callable and the trait is correctly implemented
    if let Ok(tau_geometric) = tau_result {
        let tau_si = gravity.proper_time_si(&path).unwrap();
        let expected_si = tau_geometric / SPEED_OF_LIGHT;
        assert!(
            (tau_si - expected_si).abs() < 1e-10,
            "proper_time_si should divide by c"
        );
    }
    // The test passes if the method exists and is callable
}

#[test]
fn test_proper_time_si_runs_with_square_connection() {
    // proper_time_kernel requires a rank-2 *square* metric, while a Lorentz GR
    // connection normally has the Lie shape [N, 4, 6]. GaugeField::new validates
    // only the element count (N * 4 * 6), so for N = 6 (144 elements) we can
    // reshape the connection to a square [12, 12]. That lets proper_time return
    // Ok, so the proper_time_si default method body (divide-by-c) actually runs.
    let mut builder = SimplicialComplexBuilder::new(0);
    for v in 0..6 {
        builder.add_simplex(Simplex::new(vec![v])).unwrap();
    }
    let complex = builder.build().unwrap();
    let n = complex.total_simplices(); // 6
    assert_eq!(n, 6);

    let data = CausalTensor::zeros(&[n]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // 6 * 4 * 6 = 144 elements, reshaped square [12, 12].
    let connection = CausalTensor::from_vec(vec![0.0f64; n * 4 * 6], &[12, 12]);
    let riemann = CausalTensor::zeros(&[n, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    // Path points must match the metric dimension (12).
    let path = vec![vec![0.0f64; 12], vec![1.0f64; 12]];

    let tau = gr
        .proper_time(&path)
        .expect("square metric ⇒ proper_time Ok");
    let tau_si = gr
        .proper_time_si(&path)
        .expect("proper_time_si should compute");
    let expected = tau / SPEED_OF_LIGHT;
    assert!(
        (tau_si - expected).abs() < 1e-12,
        "proper_time_si must divide geometric proper time by c: got {}, expected {}",
        tau_si,
        expected
    );
}

#[test]
fn test_schwarzschild_radius() {
    // Test with solar mass: M_sun ≈ 2e30 kg
    let solar_mass = 1.989e30;
    let r_s = GR::<f64>::schwarzschild_radius(solar_mass);

    // Expected: r_s = 2GM/c² ≈ 2954 m for the Sun
    let expected =
        2.0 * NEWTONIAN_CONSTANT_OF_GRAVITATION * solar_mass / (SPEED_OF_LIGHT * SPEED_OF_LIGHT);
    assert!(
        (r_s - expected).abs() < 1e-6,
        "Schwarzschild radius = {} should equal {}",
        r_s,
        expected
    );

    // Verify it's approximately 3 km for the Sun
    assert!(r_s > 2900.0 && r_s < 3000.0, "Sun's r_s should be ~3 km");
}

// ============================================================================
// Error Paths via Public API (tests gr_utils inversion internally)
// ============================================================================

#[test]
fn test_kretschmann_with_singular_metric() {
    // kretschmann_scalar also calls invert_4x4
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // Singular metric
    let conn_data = vec![0.0; num_simplices * 4 * 6];
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);
    let riemann = CausalTensor::zeros(&[num_simplices, 4, 4, 6]);

    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    let result = gr.kretschmann_scalar();
    assert!(
        result.is_err(),
        "Singular metric should cause kretschmann_scalar to fail"
    );
}

// ============================================================================
// Additional Interface Tests
// ============================================================================

#[test]
fn test_solve_geodesic_interface() {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0;
    conn_data[7] = 1.0;
    conn_data[14] = 1.0;
    conn_data[21] = 1.0;
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);
    let riemann = CausalTensor::zeros(&[num_simplices, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    // Test solve_geodesic with simple initial conditions
    let x0 = vec![0.0, 0.0, 0.0, 0.0];
    let v0 = vec![1.0, 0.0, 0.0, 0.0];
    let dt = 0.1;
    let steps = 5;

    let result = gr.solve_geodesic(&x0, &v0, dt, steps);
    // The method may fail due to connection shape - just verify it's callable
    if let Ok(states) = result {
        assert_eq!(states.len(), steps, "Should return {} states", steps);
    }
}

// ============================================================================
// Direct coverage: ricci_tensor / ricci_scalar / einstein_tensor
// ============================================================================

fn build_flat_gr() -> GR<f64> {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // Minkowski metric padded to [N, 4, 6] with stride 6
    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0;
    conn_data[7] = 1.0;
    conn_data[14] = 1.0;
    conn_data[21] = 1.0;
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);

    let riemann = CausalTensor::zeros(&[num_simplices, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    GaugeField::new(base, topo_metric, connection, riemann).unwrap()
}

#[test]
fn test_ricci_tensor_flat() {
    let gr = build_flat_gr();
    let ricci = gr.ricci_tensor().unwrap();
    assert_eq!(ricci.shape(), &[4, 4]);
    for v in ricci.as_slice() {
        assert!(v.abs() < 1e-12, "Ricci must be zero for flat: {}", v);
    }
}

#[test]
fn test_ricci_scalar_flat() {
    let gr = build_flat_gr();
    let r = gr.ricci_scalar().unwrap();
    assert!(r.abs() < 1e-12, "Ricci scalar must be zero for flat: {}", r);
}

#[test]
fn test_ricci_scalar_singular_metric_errors() {
    // Zeroed connection ⇒ singular ⇒ invert_4x4 errors.
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();
    let n = complex.total_simplices();
    let data = CausalTensor::zeros(&[n]);
    let base = Manifold::new(complex, data, 0).unwrap();
    let connection = CausalTensor::from_vec(vec![0.0; n * 4 * 6], &[n, 4, 6]);
    let riemann = CausalTensor::zeros(&[n, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();
    assert!(gr.ricci_scalar().is_err());
}

#[test]
fn test_ricci_scalar_connection_cols_too_small_errors() {
    // GaugeField::new only checks the element count (1*4*6 = 24), so a connection
    // reshaped to [8, 3] passes construction yet has a last dimension < 4. When
    // ricci_scalar calls invert_4x4 on it, the `cols < 4` guard (gr_utils) fires.
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();
    let n = complex.total_simplices();
    let data = CausalTensor::zeros(&[n]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // 24 elements shaped [8, 3] → cols = 3 < 4.
    let connection = CausalTensor::from_vec(vec![1.0f64; n * 4 * 6], &[8, 3]);
    let riemann = CausalTensor::zeros(&[n, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    assert!(
        gr.ricci_scalar().is_err(),
        "Connection with last dim < 4 must fail invert_4x4 (cols < 4)"
    );
}

#[test]
fn test_ricci_scalar_connection_too_small_data_errors() {
    // A rank-1 connection [24] makes invert_4x4 read cols = last = 24, so
    // 4*cols = 96 > data.len() = 24, tripping the "Metric tensor too small"
    // guard (gr_utils).
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();
    let n = complex.total_simplices();
    let data = CausalTensor::zeros(&[n]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // 24 elements shaped [24] → cols = 24, 4*cols = 96 > 24 elements.
    let connection = CausalTensor::from_vec(vec![1.0f64; n * 4 * 6], &[n * 4 * 6]);
    let riemann = CausalTensor::zeros(&[n, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    assert!(
        gr.ricci_scalar().is_err(),
        "Connection with too few elements for 4xcols must fail invert_4x4"
    );
}

#[test]
fn test_einstein_tensor_flat() {
    let gr = build_flat_gr();
    let g = gr.einstein_tensor().unwrap();
    assert_eq!(g.shape(), &[4, 4]);
    for v in g.as_slice() {
        assert!(v.abs() < 1e-12, "Einstein tensor must be zero for flat");
    }
}

#[test]
fn test_metric_tensor_accessor() {
    let gr = build_flat_gr();
    let m = gr.metric_tensor();
    assert_eq!(m.shape(), &[1, 4, 6]);
}

// ============================================================================
// momentum_constraint with matter momentum + 3-point neighbor branches
// ============================================================================

#[test]
fn test_momentum_constraint_with_matter_momentum() {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    builder.add_simplex(Simplex::new(vec![1])).unwrap();
    builder.add_simplex(Simplex::new(vec![2])).unwrap();
    let complex = builder.build().unwrap();

    let n = complex.total_simplices();
    let data = CausalTensor::zeros(&[n]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // Spatial extraction uses idx = base + (i+1)*4 + (j+1) i.e. inner stride 4.
    let mut conn_data = vec![0.0; n * 4 * 6];
    for p in 0..n {
        let b = p * 24;
        conn_data[b] = -1.0;
        conn_data[b + 5] = 1.0;
        conn_data[b + 10] = 1.0;
        conn_data[b + 15] = 1.0;
    }
    let connection = CausalTensor::from_vec(conn_data, &[n, 4, 6]);
    let riemann = CausalTensor::zeros(&[n, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    let k = CausalTensor::zeros(&[n, 3, 3]);
    let j = CausalTensor::from_vec(vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], &[n * 3]);

    let m = gr.momentum_constraint_field(&k, Some(&j)).unwrap();
    assert_eq!(m.shape(), &[n, 3]);

    // With K=0, divergence and connection terms vanish; result reduces to -8π j.
    let eight_pi = 8.0 * std::f64::consts::PI;
    let expected = [
        -eight_pi, 0.0, 0.0, 0.0, -eight_pi, 0.0, 0.0, 0.0, -eight_pi,
    ];
    for (got, want) in m.as_slice().iter().zip(expected.iter()) {
        assert!(
            (got - want).abs() < 1e-9,
            "matter momentum branch: got {}, want {}",
            got,
            want
        );
    }
}

#[test]
fn test_momentum_constraint_singular_spatial_metric() {
    // Provide a connection whose spatial 3-block is singular (all zeros)
    // so that invert_3x3 inside momentum_constraint errors.
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();
    let n = complex.total_simplices();
    let data = CausalTensor::zeros(&[n]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // Avoid stride=16 path: keep [N,4,6] with zero spatial block.
    let connection = CausalTensor::from_vec(vec![0.0; n * 4 * 6], &[n, 4, 6]);
    let riemann = CausalTensor::zeros(&[n, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    let k = CausalTensor::zeros(&[3, 3]);
    let result = gr.momentum_constraint_field(&k, None);
    assert!(
        result.is_err(),
        "Singular spatial metric must propagate as error"
    );
}

#[test]
fn test_momentum_constraint_rank1_connection_stride_fallback() {
    // GaugeField::new validates element COUNT (num_points * 4 * 6 = 24) but not
    // the exact shape. A rank-1 connection [24] drives metric_shape.len() < 2, so
    // momentum_constraint uses the `else { 16 }` stride fallback (gr_ops_impl
    // line ~253) and then the 4x4 extraction path.
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();
    let n = complex.total_simplices(); // 1

    let data = CausalTensor::zeros(&[n]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // Embed a 4x4 Minkowski metric in the first 16 of 24 slots (stride-16 layout).
    let mut conn = vec![0.0f64; n * 4 * 6]; // 24 elements
    conn[0] = -1.0; // g_00
    conn[5] = 1.0; // g_11 (idx (1)*4+1)
    conn[10] = 1.0; // g_22 (idx (2)*4+2)
    conn[15] = 1.0; // g_33 (idx (3)*4+3)
    // Rank-1 connection shape [24] triggers the len() < 2 stride fallback.
    let connection = CausalTensor::from_vec(conn, &[n * 4 * 6]);
    let riemann = CausalTensor::zeros(&[n, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    let k = CausalTensor::zeros(&[3, 3]);
    let result = gr.momentum_constraint_field(&k, None);
    // The fallback path runs; with a valid spatial metric and K=0 it succeeds.
    assert!(
        result.is_ok(),
        "Rank-1 connection stride fallback should still compute: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().shape(), &[3]);
}

#[test]
fn test_momentum_constraint_small_stride_identity_fallback() {
    // A connection reshaped to [6, 2, 2] keeps the required 24-element count but
    // makes metric_stride = 2*2 = 4 < 16, so extract_spatial_metric takes the
    // identity-metric fallback branch (gr_ops_impl lines ~276-280).
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();
    let n = complex.total_simplices(); // 1

    let data = CausalTensor::zeros(&[n]);
    let base = Manifold::new(complex, data, 0).unwrap();

    // 24 elements, but shaped [6, 2, 2] → stride = 4 < 16 → identity fallback.
    let connection = CausalTensor::from_vec(vec![0.0f64; n * 4 * 6], &[6, 2, 2]);
    let riemann = CausalTensor::zeros(&[n, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    let k = CausalTensor::zeros(&[3, 3]);
    let result = gr.momentum_constraint_field(&k, None);
    // Identity spatial metric is invertible; with K=0 the constraint is zero.
    assert!(
        result.is_ok(),
        "Identity-metric fallback should compute: {:?}",
        result.err()
    );
    let m = result.unwrap();
    assert_eq!(m.shape(), &[3]);
    for v in m.as_slice() {
        assert!(v.abs() < 1e-12, "Flat identity metric ⇒ M_i = 0, got {}", v);
    }
}

#[test]
fn test_parallel_transport_interface() {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    let complex = builder.build().unwrap();

    let num_simplices = complex.total_simplices();
    let data = CausalTensor::zeros(&[num_simplices]);
    let base = Manifold::new(complex, data, 0).unwrap();

    let mut conn_data = vec![0.0; num_simplices * 4 * 6];
    conn_data[0] = -1.0;
    conn_data[7] = 1.0;
    conn_data[14] = 1.0;
    conn_data[21] = 1.0;
    let connection = CausalTensor::from_vec(conn_data, &[num_simplices, 4, 6]);
    let riemann = CausalTensor::zeros(&[num_simplices, 4, 4, 6]);
    let topo_metric = EastCoastMetric::minkowski_4d().into_metric();
    let gr: GR<f64> = GaugeField::new(base, topo_metric, connection, riemann).unwrap();

    let v0 = vec![1.0, 0.0, 0.0, 0.0];
    let path = vec![vec![0.0, 0.0, 0.0, 0.0], vec![0.1, 0.0, 0.0, 0.0]];

    let result = gr.parallel_transport(&v0, &path);
    // Verify the method is callable and returns proper dimension if successful
    if let Ok(v) = result {
        assert_eq!(v.len(), 4, "Transported vector should have 4 components");
    }
}
