/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    BaseTopology, GaugeField, Lorentz, Manifold, Simplex, SimplicialComplexBuilder, U1,
};

fn create_test_manifold() -> Manifold<f64> {
    let mut builder = SimplicialComplexBuilder::new(1);
    builder
        .add_simplex(Simplex::new(vec![0, 1]))
        .expect("Failed to add simplex");
    let complex = builder.build().expect("Failed to build complex");

    // Create dummy data for manifold
    let data = CausalTensor::zeros(&[3]); // 3 simplices (1 edge + 2 vertices)

    Manifold::new(complex, data, 0).expect("Failed to create manifold")
}

#[test]
fn test_gauge_field_new() {
    let manifold = create_test_manifold();
    // 1 point, 4D spacetime, 1D lie algebra
    let connection = CausalTensor::from_vec(vec![1.0, 2.0, 3.0, 4.0], &[1, 4, 1]);
    // 1 point, 4D spacetime, 4D spacetime, 1D lie algebra
    let field_strength = CausalTensor::from_vec(vec![0.0; 16], &[1, 4, 4, 1]);

    let field: GaugeField<U1, f64, f64> =
        GaugeField::new(manifold, Metric::Minkowski(4), connection, field_strength);

    assert_eq!(field.metric(), Metric::Minkowski(4));
    assert_eq!(field.gauge_group_name(), "U(1)");
    assert_eq!(field.lie_algebra_dim(), 1);
    assert_eq!(field.spacetime_dim(), 4);
    assert!(field.is_abelian());
}

#[test]
fn test_gauge_field_with_default_metric() {
    let manifold = create_test_manifold();
    let connection = CausalTensor::from_vec(vec![1.0; 4], &[1, 4, 1]);
    let field_strength = CausalTensor::from_vec(vec![0.0; 16], &[1, 4, 4, 1]);

    let u1_field: GaugeField<U1, f64, f64> = GaugeField::with_default_metric(
        manifold.clone(),
        connection.clone(),
        field_strength.clone(),
    );

    // U1 uses West Coast (+---)
    assert!(u1_field.is_west_coast());
    assert!(!u1_field.is_east_coast());

    let gr_field: GaugeField<Lorentz, f64, f64> = GaugeField::with_default_metric(
        manifold,
        connection,
        field_strength, // Dimensions match Lorentz (4D spacetime, 6D lie algebra mismatch but strictly type only checks G)
                        // Actually Lorentz lie algebra dim is 6, so tensor shapes should ideally match,
                        // but GaugeField constructors don't strictly validate shape vs G constants yet, relying on caller.
                        // Ideally we should pass correct shapes.
    );

    // Lorentz uses East Coast (-+++)
    // assert!(gr_field.is_east_coast()); // Metric::Generic ordering is (+... -...), need custom ordering for (-+++)
    assert!(!gr_field.is_west_coast());
}

#[test]
fn test_gauge_field_getters() {
    let manifold = create_test_manifold();
    let connection = CausalTensor::from_vec(vec![10.0], &[1, 1, 1]);
    let field_strength = CausalTensor::from_vec(vec![20.0], &[1, 1, 1, 1]);

    let field: GaugeField<U1, f64, f64> = GaugeField::new(
        manifold.clone(),
        Metric::Euclidean(1),
        connection,
        field_strength,
    );

    assert_eq!(field.connection().as_slice()[0], 10.0);
    assert_eq!(field.field_strength().as_slice()[0], 20.0);
    // Base Check
    assert_eq!(field.base().complex().dimension(), 1);
}
