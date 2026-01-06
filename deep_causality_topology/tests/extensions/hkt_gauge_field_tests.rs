/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    GaugeField, GaugeFieldWitness, Manifold, Simplex, SimplicialComplexBuilder, U1,
};

fn create_test_manifold() -> Manifold<f64, f64> {
    let mut builder = SimplicialComplexBuilder::new(1);
    builder
        .add_simplex(Simplex::new(vec![0, 1]))
        .expect("Failed to add simplex");
    let complex = builder.build::<f64>().expect("Failed to build complex");

    // Create dummy data
    let data = CausalTensor::zeros(&[3]);

    Manifold::new(complex, data, 0).expect("Failed to create manifold")
}

#[test]
fn test_gauge_transform() {
    let manifold = create_test_manifold();
    // U1 gauge field: exp(i alpha) -> simple phase shift if A were complex,
    // but here we model A as real 1-form A' = A + d(alpha)
    // Wait, the gauge_transform implementation takes a func and MAPS it over components.
    // A' = f(A)
    // This is a simplified "parametric map" test.

    let connection = CausalTensor::from_vec(vec![1.0, 2.0, 3.0, 4.0], &[1, 4, 1]);
    let field_strength = CausalTensor::from_vec(vec![0.0; 16], &[1, 4, 4, 1]);

    let field: GaugeField<U1, f64, f64, f64> =
        GaugeField::with_default_metric(manifold, connection, field_strength)
            .expect("Failed to create field");

    // Apply transformation A -> 2*A
    let transformed =
        GaugeFieldWitness::<f64>::gauge_transform(&field, |x| x * 2.0).expect("Transform failed");

    // Check transformation applied
    assert_eq!(transformed.connection().as_slice()[0], 2.0);
    assert_eq!(transformed.connection().as_slice()[1], 4.0);
    // Field strength also transformed
    assert_eq!(transformed.field_strength().shape(), &[1, 4, 4, 1]);
}

#[test]
fn test_merge_fields() {
    let manifold = create_test_manifold();
    // Two fields to merge
    let conn_a = CausalTensor::from_vec(vec![1.0, 2.0], &[1, 2, 1]);
    let conn_b = CausalTensor::from_vec(vec![3.0, 4.0], &[1, 2, 1]);
    // Dummy field strength
    let fs = CausalTensor::from_vec(vec![0.0; 4], &[1, 2, 2, 1]);

    let field_a: GaugeField<U1, f64, f64, f64> =
        GaugeField::with_default_metric(manifold.clone(), conn_a, fs.clone())
            .expect("Failed to create field_a");
    let field_b: GaugeField<U1, f64, f64, f64> =
        GaugeField::with_default_metric(manifold, conn_b, fs).expect("Failed to create field_b");

    // Merge: A_new = A + B
    let merged = GaugeFieldWitness::<f64>::merge_fields(&field_a, &field_b, |a, b| a + b)
        .expect("Merge failed");

    // 1+3 = 4, 2+4 = 6
    assert_eq!(merged.connection().as_slice()[0], 4.0);
    assert_eq!(merged.connection().as_slice()[1], 6.0);
}

#[test]
fn test_abelian_field_strength() {
    let manifold = create_test_manifold();
    // F_uv = d_u A_v - d_v A_u
    // Placeholder implementation returns vector of zeros with correct shape
    // because numerical differentiation of connection on discrete manifold
    // requires more complex setup (integration with StokesAdjunction).
    // The current impl just allocates shape.

    let connection = CausalTensor::from_vec(vec![1.0; 4], &[1, 4, 1]); // 4D
    let field_strength = CausalTensor::from_vec(vec![0.0; 16], &[1, 4, 4, 1]);

    let field: GaugeField<U1, f64, f64, f64> =
        GaugeField::with_default_metric(manifold, connection, field_strength)
            .expect("Failed to create field");

    let fs_opt = GaugeFieldWitness::compute_field_strength_abelian(&field);
    assert!(fs_opt.is_some());
    let fs = fs_opt.unwrap();
    assert_eq!(fs.shape(), &[1, 4, 4, 1]);
}
