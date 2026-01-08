/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    GaugeField, GaugeGroup, Lorentz, Manifold, Simplex, SimplicialComplexBuilder, TopologyError,
    TopologyErrorEnum, U1,
};

// Helper to create valid inputs
fn create_test_data() -> (Manifold<f64, f64>, CausalTensor<f64>) {
    // Create a 0D manifold (single point) which is a valid manifold
    let mut builder = SimplicialComplexBuilder::new(0);
    builder.add_simplex(Simplex::new(vec![0])).expect("Adding simplex");
    let complex = builder.build::<f64>().expect("Building complex");
    
    // 1 simplex (vertex 0)
    let data = CausalTensor::zeros(&[1]);
    let manifold = Manifold::new(complex, data, 0).expect("Manifold");

    // Valid tensor dummy for reusing
    let tensor = CausalTensor::zeros(&[1]);
    (manifold, tensor)
}

// <...>

#[test]
fn test_gauge_field_new_connection_mismatch() {
    let (manifold, _) = create_test_data();
    // U1: spacetime_dim=4, lie_dim=1. Total elements = num_points(1) * 4 * 1 = 4.
    // Provide tensor with just 2 elements.
    let connection = CausalTensor::zeros(&[1, 2, 1]); // Wrong shape
    let field_strength = CausalTensor::zeros(&[1, 4, 4, 1]); // Correct shape

    let result: Result<GaugeField<U1, f64, f64, f64>, _> = GaugeField::new(
        manifold,
        Metric::Minkowski(4),
        connection,
        field_strength,
    );

    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::GaugeFieldError(msg))) => {
            assert!(msg.contains("Connection shape mismatch"));
        }
        _ => panic!("Expected GaugeFieldError"),
    }
}

#[test]
fn test_gauge_field_new_field_strength_mismatch() {
    let (manifold, _) = create_test_data();
    // U1: num*4*4*1 = 16 elements.
    let connection = CausalTensor::zeros(&[1, 4, 1]); // Correct
    let field_strength = CausalTensor::zeros(&[1, 2, 2, 1]); // Wrong shape/elements => 4 elements

    let result: Result<GaugeField<U1, f64, f64, f64>, _> = GaugeField::new(
        manifold,
        Metric::Minkowski(4),
        connection,
        field_strength,
    );

    assert!(result.is_err());
    match result {
        Err(TopologyError(TopologyErrorEnum::GaugeFieldError(msg))) => {
            assert!(msg.contains("Field strength shape mismatch"));
        }
        _ => panic!("Expected GaugeFieldError"),
    }
}

// Dummy Gauge Group for testing generic properties or defaults
#[derive(Clone, Debug)]
struct TestGroup;
impl GaugeGroup for TestGroup {
    const LIE_ALGEBRA_DIM: usize = 1;
    const IS_ABELIAN: bool = true;
    fn name() -> &'static str {
        "TestGroup"
    }
}

#[test]
fn test_gauge_group_defaults() {
    // Default structure constant should be 0.0
    assert_eq!(TestGroup::structure_constant(0, 0, 0), 0.0);
}

// We rely on existing tests for is_west_coast/is_east_coast which use U1 and Lorentz
// But we can add a specific test for custom metric signatures if needed.

#[test]
fn test_metric_conventions_custom() {
    // Manually test the logic:
    // West Coast: (+---)
    // East Coast: (-+++)

    let (manifold, _) = create_test_data();
    let conn = CausalTensor::zeros(&[1, 4, 1]);
    let fs = CausalTensor::zeros(&[1, 4, 4, 1]);

    // Construct with Explicit West Coast (+---)
    // Metric::Minkowski(4) typically defaults to one convention.
    // Let's rely on what we get and verify consistency.
    let metric = Metric::Minkowski(4);
    
    let field = GaugeField::<U1, f64, f64, f64>::new(
        manifold,
        metric,
        conn,
        fs,
    ).expect("Field");

    // Check signatures directly
    let s0 = field.metric().sign_of_sq(0);
    let s1 = field.metric().sign_of_sq(1);

    if s0 == 1 && s1 == -1 {
         assert!(field.is_west_coast());
         assert!(!field.is_east_coast());
    } else if s0 == -1 && s1 == 1 {
         assert!(field.is_east_coast());
         assert!(!field.is_west_coast());
    } else {
        // Unknown or Euclidean
        assert!(!field.is_east_coast());
        assert!(!field.is_west_coast());
    }
}
