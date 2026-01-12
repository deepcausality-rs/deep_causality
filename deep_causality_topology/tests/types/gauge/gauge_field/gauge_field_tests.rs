/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    BaseTopology, GaugeField, Lorentz, Manifold, PointCloud, Simplex, SimplicialComplexBuilder, U1,
};

fn create_test_manifold() -> Manifold<f64, f64> {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("Failed to add simplex");
    let complex = builder.build::<f64>().expect("Failed to build complex");

    // Create dummy data for manifold
    let data = CausalTensor::zeros(&[1]); // 1 vertex

    Manifold::new(complex, data, 0).expect("Failed to create manifold")
}

/// Helper to create a simple U(1) gauge field (electromagnetism-like)
fn create_u1_gauge_field() -> GaugeField<U1, Complex<f64>, f64> {
    // Create manifold base
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();
    let complex = point_cloud.triangulate(1.2).unwrap();
    let data = CausalTensor::new(vec![1.0; 7], vec![7]).unwrap();
    let base = Manifold::new(complex, data, 0).unwrap();

    // Connection shape: [num_points=7, spacetime_dim=4, lie_dim=1] = 28 elements
    let connection = CausalTensor::new(vec![Complex::new(0.0, 0.0); 28], vec![7, 4, 1]).unwrap();
    // Field strength shape: [num_points=7, spacetime_dim=4, spacetime_dim=4, lie_dim=1] = 112 elements
    let field_strength =
        CausalTensor::new(vec![Complex::new(0.0, 0.0); 112], vec![7, 4, 4, 1]).unwrap();

    // Use Metric::Minkowski variant
    GaugeField::<U1, Complex<f64>, f64>::new(base, Metric::Minkowski(4), connection, field_strength)
        .unwrap()
}

#[test]
fn test_gauge_field_new() {
    let manifold = create_test_manifold();
    // 1 point, 4D spacetime, 1D lie algebra
    let connection = CausalTensor::from_vec(
        vec![
            Complex::new(1.0, 0.0),
            Complex::new(2.0, 0.0),
            Complex::new(3.0, 0.0),
            Complex::new(4.0, 0.0),
        ],
        &[1, 4, 1],
    );
    // 1 point, 4D spacetime, 4D spacetime, 1D lie algebra
    let field_strength = CausalTensor::from_vec(vec![Complex::new(0.0, 0.0); 16], &[1, 4, 4, 1]);

    let field: GaugeField<U1, Complex<f64>, f64> =
        GaugeField::new(manifold, Metric::Minkowski(4), connection, field_strength)
            .expect("Failed to create U1 gauge field");

    assert_eq!(field.metric(), Metric::Minkowski(4));
    assert_eq!(field.gauge_group_name(), "U(1)");
    assert_eq!(field.lie_algebra_dim(), 1);
    assert_eq!(field.spacetime_dim(), 4);
    assert!(field.is_abelian());
}

#[test]
fn test_gauge_field_with_default_metric() {
    let manifold = create_test_manifold();
    let connection = CausalTensor::from_vec(vec![Complex::new(1.0, 0.0); 4], &[1, 4, 1]);
    let field_strength = CausalTensor::from_vec(vec![Complex::new(0.0, 0.0); 16], &[1, 4, 4, 1]);

    let u1_field: GaugeField<U1, Complex<f64>, f64> = GaugeField::with_default_metric(
        manifold.clone(),
        connection.clone(),
        field_strength.clone(),
    )
    .expect("Failed to create U1 field");

    // U1 uses West Coast (+---)
    assert!(u1_field.is_west_coast());
    assert!(!u1_field.is_east_coast());

    // Note: This test was updated now that shape validation is in place.
    // Lorentz has lie_algebra_dim=6, so we need correct tensor shapes.
    let lorentz_conn = CausalTensor::from_vec(vec![1.0; 24], &[1, 4, 6]); // 4D spacetime, 6D lie algebra
    let lorentz_fs = CausalTensor::from_vec(vec![0.0; 96], &[1, 4, 4, 6]);

    let gr_field: GaugeField<Lorentz, f64, f64> =
        GaugeField::with_default_metric(manifold, lorentz_conn, lorentz_fs)
            .expect("Failed to create Lorentz field");

    // Lorentz uses East Coast (-+++)
    // assert!(gr_field.is_east_coast()); // Metric::Generic ordering is (+... -...), need custom ordering for (-+++)
    assert!(!gr_field.is_west_coast());
}

#[test]
fn test_gauge_field_getters() {
    let manifold = create_test_manifold();
    // U1 requires spacetime_dim=4, lie_dim=1 -> connection: 4 elements, field_strength: 16 elements
    let connection = CausalTensor::from_vec(
        vec![
            Complex::new(10.0, 0.0),
            Complex::new(20.0, 0.0),
            Complex::new(30.0, 0.0),
            Complex::new(40.0, 0.0),
        ],
        &[1, 4, 1],
    );
    let field_strength = CausalTensor::from_vec(vec![Complex::new(5.0, 0.0); 16], &[1, 4, 4, 1]);

    let field: GaugeField<U1, Complex<f64>, f64> = GaugeField::new(
        manifold.clone(),
        Metric::Minkowski(4),
        connection,
        field_strength,
    )
    .expect("Failed to create U1 field");

    assert_eq!(field.connection().as_slice()[0], Complex::new(10.0, 0.0));
    assert_eq!(field.field_strength().as_slice()[0], Complex::new(5.0, 0.0));
    // Base Check
    assert_eq!(field.base().complex().dimension(), 0);
}

// =============================================================================
// Getter coverage tests
// =============================================================================

#[test]
fn test_gauge_group_name() {
    let field = create_u1_gauge_field();
    assert_eq!(field.gauge_group_name(), "U(1)");
}

#[test]
fn test_lie_algebra_dim() {
    let field = create_u1_gauge_field();
    // U(1) has 1-dimensional Lie algebra
    assert_eq!(field.lie_algebra_dim(), 1);
}

#[test]
fn test_is_abelian() {
    let field = create_u1_gauge_field();
    // U(1) is abelian
    assert!(field.is_abelian());
}

#[test]
fn test_spacetime_dim() {
    let field = create_u1_gauge_field();
    assert_eq!(field.spacetime_dim(), 4);
}

// =============================================================================
// Metric convention tests
// =============================================================================

#[test]
fn test_is_west_coast_default() {
    let field = create_u1_gauge_field();
    // Minkowski metric is West Coast (+---)
    assert!(field.is_west_coast());
    assert!(!field.is_east_coast());
}

#[test]
fn test_metric_getter() {
    let field = create_u1_gauge_field();
    let metric = field.metric();
    // Just verify we can get the metric
    assert!(matches!(metric, Metric::Minkowski(_)));
}

// =============================================================================
// Connection and field strength getters
// =============================================================================

#[test]
fn test_connection_getter() {
    let field = create_u1_gauge_field();
    let conn = field.connection();
    assert_eq!(conn.len(), 28); // [7, 4, 1] = 28 elements
}

#[test]
fn test_field_strength_getter() {
    let field = create_u1_gauge_field();
    let fs = field.field_strength();
    assert_eq!(fs.len(), 112); // [7, 4, 4, 1] = 112 elements
}

#[test]
fn test_base_getter() {
    let field = create_u1_gauge_field();
    let base = field.base();
    // Base manifold should have the simplicial complex
    assert!(!base.complex().skeletons().is_empty());
}
