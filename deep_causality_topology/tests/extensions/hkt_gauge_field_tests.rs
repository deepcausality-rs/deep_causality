/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{ParametricMonad, Promonad};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    GaugeField, GaugeFieldHKT, GaugeFieldWitness, Manifold, SU2, Simplex, SimplicialComplexBuilder,
    U1,
};

fn create_test_manifold() -> Manifold<f64, f64> {
    let mut builder = SimplicialComplexBuilder::new(0);
    builder
        .add_simplex(Simplex::new(vec![0]))
        .expect("Failed to add simplex");
    let complex = builder.build::<f64>().expect("Failed to build complex");

    // Create dummy data
    let data = CausalTensor::zeros(&[1]);

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

    let field: GaugeField<U1, f64, f64> =
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
    // Two fields to merge - U1 requires spacetime_dim=4
    let conn_a = CausalTensor::from_vec(vec![1.0, 2.0, 3.0, 4.0], &[1, 4, 1]);
    let conn_b = CausalTensor::from_vec(vec![5.0, 6.0, 7.0, 8.0], &[1, 4, 1]);
    // Dummy field strength
    let fs = CausalTensor::from_vec(vec![0.0; 16], &[1, 4, 4, 1]);

    let field_a: GaugeField<U1, f64, f64> =
        GaugeField::with_default_metric(manifold.clone(), conn_a, fs.clone())
            .expect("Failed to create field_a");
    let field_b: GaugeField<U1, f64, f64> =
        GaugeField::with_default_metric(manifold, conn_b, fs).expect("Failed to create field_b");

    // Merge: A_new = A + B
    let merged = GaugeFieldWitness::<f64>::merge_fields(&field_a, &field_b, |a, b| a + b)
        .expect("Merge failed");

    // 1+5 = 6, 2+6 = 8
    assert_eq!(merged.connection().as_slice()[0], 6.0);
    assert_eq!(merged.connection().as_slice()[1], 8.0);
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

    let field: GaugeField<U1, f64, f64> =
        GaugeField::with_default_metric(manifold, connection, field_strength)
            .expect("Failed to create field");

    let fs_opt = GaugeFieldWitness::compute_field_strength_abelian(&field);
    assert!(fs_opt.is_some());
    let fs = fs_opt.unwrap();
    assert_eq!(fs.shape(), &[1, 4, 4, 1]);
}

// ============================================================================
// Additional HKT Tests for Coverage
// ============================================================================

#[test]
fn test_gauge_field_hkt_empty() {
    let hkt: GaugeFieldHKT<(), (), (), f64> = GaugeFieldHKT::empty();
    assert!(!hkt.has_data());
    assert!(hkt.connection_data().is_none());
    assert!(hkt.field_strength_data().is_none());
}

#[test]
fn test_gauge_field_hkt_default() {
    let hkt: GaugeFieldHKT<i32, i32, i32, f64> = GaugeFieldHKT::default();
    assert!(!hkt.has_data());
}

#[test]
fn test_gauge_field_hkt_from_data() {
    let conn_data = vec![1.0, 2.0, 3.0, 4.0];
    let fs_data = vec![0.5, 0.6, 0.7, 0.8];
    let conn_shape = vec![1, 4, 1];
    let fs_shape = vec![1, 2, 2, 1];

    let hkt: GaugeFieldHKT<(), (), (), f64> =
        GaugeFieldHKT::from_data(conn_data.clone(), fs_data.clone(), conn_shape, fs_shape);

    assert!(hkt.has_data());
    assert_eq!(hkt.connection_data(), Some(conn_data.as_slice()));
    assert_eq!(hkt.field_strength_data(), Some(fs_data.as_slice()));
}

#[test]
fn test_promonad_merge() {
    // Create two HKT wrappers with data
    let pa: GaugeFieldHKT<f64, f64, f64, f64> = GaugeFieldHKT::from_data(
        vec![1.0, 2.0],
        vec![0.5, 0.5],
        vec![1, 2, 1],
        vec![1, 1, 1, 2],
    );
    let pb: GaugeFieldHKT<f64, f64, f64, f64> = GaugeFieldHKT::from_data(
        vec![3.0, 4.0],
        vec![1.5, 1.5],
        vec![1, 2, 1],
        vec![1, 1, 1, 2],
    );

    // Merge averages the data
    let merged: GaugeFieldHKT<f64, f64, f64, f64> =
        GaugeFieldWitness::merge(pa, pb, |a: f64, b: f64| a + b);

    assert!(merged.has_data());
    // (1+3)/2=2, (2+4)/2=3
    let conn = merged.connection_data().unwrap();
    assert_eq!(conn[0], 2.0);
    assert_eq!(conn[1], 3.0);
    // (0.5+1.5)/2=1.0
    let fs = merged.field_strength_data().unwrap();
    assert_eq!(fs[0], 1.0);
}

#[test]
fn test_promonad_merge_empty() {
    let empty: GaugeFieldHKT<f64, f64, f64, f64> = GaugeFieldHKT::empty();
    let with_data: GaugeFieldHKT<f64, f64, f64, f64> =
        GaugeFieldHKT::from_data(vec![1.0], vec![2.0], vec![1, 1, 1], vec![1, 1, 1, 1]);

    // Merge with empty returns empty
    let result: GaugeFieldHKT<f64, f64, f64, f64> =
        GaugeFieldWitness::merge(empty, with_data, |a: f64, b: f64| a + b);
    assert!(!result.has_data());
}

#[test]
fn test_promonad_fuse() {
    // Fuse creates an empty wrapper (type erasure limitation)
    let result: GaugeFieldHKT<i32, f64, String, f64> = GaugeFieldWitness::fuse(42i32, 3.0f64);
    assert!(!result.has_data());
}

#[test]
fn test_parametric_monad_pure() {
    // Pure returns empty wrapper (cannot store arbitrary A)
    let result: GaugeFieldHKT<(), (), i32, f64> = GaugeFieldWitness::pure(42);
    assert!(!result.has_data());
}

#[test]
fn test_parametric_monad_ibind_empty() {
    let empty: GaugeFieldHKT<i32, i32, f64, f64> = GaugeFieldHKT::empty();

    let result: GaugeFieldHKT<i32, i64, String, f64> =
        GaugeFieldWitness::ibind(empty, |_x: f64| GaugeFieldHKT::empty());

    assert!(!result.has_data());
}

#[test]
fn test_parametric_monad_ibind_with_data() {
    let hkt: GaugeFieldHKT<i32, i32, f64, f64> = GaugeFieldHKT::from_data(
        vec![1.0, 2.0],
        vec![3.0, 4.0],
        vec![1, 2, 1],
        vec![1, 1, 1, 2],
    );

    let result: GaugeFieldHKT<i32, i64, String, f64> =
        GaugeFieldWitness::ibind(hkt, |_x: f64| GaugeFieldHKT::empty());

    // ibind propagates data unchanged (placeholder impl)
    assert!(result.has_data());
    assert_eq!(result.connection_data().unwrap(), &[1.0, 2.0]);
}

#[test]
fn test_compute_field_strength_non_abelian() {
    let manifold = create_test_manifold();

    // SU(2) gauge field
    let lie_dim = 3; // su(2) has 3 generators
    let dim = 4;
    let num_points = 1;

    // Connection shape: [num_points, dim, lie_dim]
    let conn_data: Vec<f64> = (0..(num_points * dim * lie_dim))
        .map(|i| (i as f64) * 0.1)
        .collect();
    let connection = CausalTensor::from_vec(conn_data, &[num_points, dim, lie_dim]);

    // Field strength shape: [num_points, dim, dim, lie_dim]
    let fs = CausalTensor::zeros(&[num_points, dim, dim, lie_dim]);

    let field: GaugeField<SU2, f64, f64> =
        GaugeField::with_default_metric(manifold, connection, fs).expect("Failed to create field");

    // Compute with coupling constant g = 1.0
    let result = GaugeFieldWitness::compute_field_strength_non_abelian(&field, 1.0);

    // Verify shape
    assert_eq!(result.shape(), &[num_points, dim, dim, lie_dim]);
    // With non-zero structure constants and connection, we should have non-zero field strength
    assert!(result.as_slice().iter().any(|&x| x != 0.0));
}

#[test]
fn test_field_strength_from_eb_vectors() {
    let e = [1.0f64, 2.0, 3.0]; // Electric field
    let b = [0.1f64, 0.2, 0.3]; // Magnetic field

    let fs = GaugeFieldWitness::field_strength_from_eb_vectors(&e, &b, 1);

    // Shape [1, 4, 4, 1] for U(1) in 4D
    assert_eq!(fs.shape(), &[1, 4, 4, 1]);

    let data = fs.as_slice();
    // F_01 = E_x = 1.0
    assert_eq!(data[1], 1.0);
    // F_10 = -E_x = -1.0
    assert_eq!(data[4], -1.0);
    // F_02 = E_y = 2.0
    assert_eq!(data[2], 2.0);
    // F_03 = E_z = 3.0
    assert_eq!(data[3], 3.0);
    // F_23 (index 11) = B_x = 0.1
    assert_eq!(data[11], 0.1);
}

#[test]
fn test_field_strength_from_eb_multiple_points() {
    let e = [1.0f64, 0.0, 0.0];
    let b = [0.0f64, 0.0, 1.0];

    let fs = GaugeFieldWitness::field_strength_from_eb_vectors(&e, &b, 3);

    // Shape [3, 4, 4, 1]
    assert_eq!(fs.shape(), &[3, 4, 4, 1]);
    // Each point should have the same field values
    let data = fs.as_slice();
    // Point 0: F_01 = 1.0
    assert_eq!(data[1], 1.0);
    // Point 1: F_01 = 1.0 (offset = 16)
    assert_eq!(data[16 + 1], 1.0);
    // Point 2: F_01 = 1.0 (offset = 32)
    assert_eq!(data[32 + 1], 1.0);
}

#[test]
#[should_panic(expected = "Electric field must have 3 components")]
fn test_field_strength_from_eb_invalid_e() {
    let e = [1.0f64, 2.0]; // Wrong size
    let b = [0.1f64, 0.2, 0.3];
    GaugeFieldWitness::field_strength_from_eb_vectors(&e, &b, 1);
}

#[test]
#[should_panic(expected = "Magnetic field must have 3 components")]
fn test_field_strength_from_eb_invalid_b() {
    let e = [1.0f64, 2.0, 3.0];
    let b = [0.1f64, 0.2]; // Wrong size
    GaugeFieldWitness::field_strength_from_eb_vectors(&e, &b, 1);
}

#[test]
fn test_gauge_rotation() {
    // Create connection and field strength tensors with proper shape
    let num_points = 1;
    let dim = 4;
    let lie_dim = 4; // Electroweak has 4 components (W1, W2, W3, B)

    // Connection: [1, 4, 4]
    let mut conn_data = vec![0.0f64; num_points * dim * lie_dim];
    // Set W3 (index 2) and B (index 3) components
    for mu in 0..dim {
        conn_data[mu * lie_dim + 2] = 1.0; // W3_mu = 1.0
        conn_data[mu * lie_dim + 3] = 2.0; // B_mu = 2.0
    }
    let connection = CausalTensor::from_vec(conn_data, &[num_points, dim, lie_dim]);

    // Field strength: [1, 4, 4, 4]
    let mut fs_data = vec![0.0f64; num_points * dim * dim * lie_dim];
    for mu in 0..dim {
        for nu in 0..dim {
            let idx = mu * (dim * lie_dim) + nu * lie_dim;
            fs_data[idx + 2] = 0.5; // F^W3_μν
            fs_data[idx + 3] = 0.7; // F^B_μν
        }
    }
    let field_strength = CausalTensor::from_vec(fs_data, &[num_points, dim, dim, lie_dim]);

    // Weinberg angle mixing: cos(θ_W) ≈ 0.88, sin(θ_W) ≈ 0.48
    let cos_w = 0.88;
    let sin_w = 0.48;

    let (rotated_conn, rotated_fs) = GaugeFieldWitness::gauge_rotation(
        &connection,
        &field_strength,
        2, // W3 index
        3, // B index
        cos_w,
        sin_w,
    );

    // New connection should have lie_dim = 1
    assert_eq!(rotated_conn.shape(), &[num_points, dim, 1]);
    // New field strength should have lie_dim = 1
    assert_eq!(rotated_fs.shape(), &[num_points, dim, dim, 1]);

    // A'_μ = W3_μ * sin(θ) + B_μ * cos(θ) = 1.0 * 0.48 + 2.0 * 0.88 = 2.24
    let expected = 1.0 * sin_w + 2.0 * cos_w;
    assert!((rotated_conn.as_slice()[0] - expected).abs() < 1e-10);
}

#[test]
fn test_gauge_rotation_invalid_shapes() {
    // Connection with too few dimensions
    let conn = CausalTensor::from_vec(vec![1.0, 2.0], &[2]);
    let fs = CausalTensor::from_vec(vec![1.0], &[1]);

    let (rotated_conn, rotated_fs) = GaugeFieldWitness::gauge_rotation(&conn, &fs, 0, 1, 1.0, 0.0);

    // Should return empty tensors
    assert_eq!(rotated_conn.shape(), &[0]);
    assert_eq!(rotated_fs.shape(), &[0]);
}

#[test]
fn test_gauge_rotation_invalid_indices() {
    // Valid shapes but indices out of bounds
    let conn = CausalTensor::from_vec(vec![1.0; 8], &[1, 4, 2]);
    let fs = CausalTensor::from_vec(vec![1.0; 32], &[1, 4, 4, 2]);

    // index_a = 5 is out of bounds for lie_dim = 2
    let (rotated_conn, rotated_fs) = GaugeFieldWitness::gauge_rotation(&conn, &fs, 5, 0, 1.0, 0.0);

    // Should return empty tensors
    assert_eq!(rotated_conn.shape(), &[0]);
    assert_eq!(rotated_fs.shape(), &[0]);
}
