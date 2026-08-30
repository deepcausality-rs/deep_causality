/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for SimplicialComplex public `boundary_operator` / `coboundary_operator` APIs
//! in `src/types/simplicial_complex/api/boundary.rs` and the underlying
//! `src/types/simplicial_complex/boundary/boundary_impl.rs`.

use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{
    Simplex, SimplicialComplexBuilder, TopologyError, TopologyErrorEnum,
};

#[test]
fn test_boundary_operator_k0_returns_error() {
    let complex = create_triangle_complex();
    let result: Result<_, TopologyError> = complex.boundary_operator(0);
    assert!(result.is_err(), "boundary_operator(0) must error");
    match result.unwrap_err().0 {
        TopologyErrorEnum::DimensionMismatch(ref msg) => {
            assert!(
                msg.contains("dimension 0"),
                "Error message should mention dimension 0: {}",
                msg
            );
        }
        ref e => panic!("Expected DimensionMismatch, got {:?}", e),
    }
}

#[test]
fn test_boundary_operator_valid_dimensions() {
    let complex = create_triangle_complex();

    // k = 1: edges -> vertices. 3 vertices, 3 edges.
    let b1 = complex.boundary_operator(1).unwrap();
    assert_eq!(b1.shape(), (3, 3));

    // k = 2: faces -> edges. 3 edges, 1 face.
    let b2 = complex.boundary_operator(2).unwrap();
    assert_eq!(b2.shape(), (3, 1));
}

#[test]
fn test_boundary_operator_out_of_range_errors() {
    let complex = create_triangle_complex();
    let result = complex.boundary_operator(10);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        TopologyErrorEnum::DimensionMismatch(ref msg) => {
            assert!(msg.contains("dimension 10"), "got {}", msg);
        }
        ref e => panic!("Expected DimensionMismatch, got {:?}", e),
    }
}

#[test]
fn test_coboundary_operator_empty_when_helper_creates_complex() {
    // create_triangle_complex builds without coboundary operators.
    let complex = create_triangle_complex();
    let result = complex.coboundary_operator(0);
    assert!(
        result.is_err(),
        "coboundary_operator(0) should error when none stored"
    );
    match result.unwrap_err().0 {
        TopologyErrorEnum::DimensionMismatch(_) => {}
        ref e => panic!("Expected DimensionMismatch, got {:?}", e),
    }
}

#[test]
fn test_coboundary_operator_via_builder_built_complex() {
    // The builder pipeline populates coboundary operators.
    let mut builder = SimplicialComplexBuilder::new(1);
    builder.add_simplex(Simplex::new(vec![0])).unwrap();
    builder.add_simplex(Simplex::new(vec![1])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    // Should have a coboundary at dim 0.
    let cob0 = complex.coboundary_operator(0);
    assert!(cob0.is_ok(), "coboundary_operator(0) should succeed");

    // Out-of-range coboundary returns error.
    let cob_huge = complex.coboundary_operator(99);
    assert!(cob_huge.is_err());
    match cob_huge.unwrap_err().0 {
        TopologyErrorEnum::DimensionMismatch(ref msg) => {
            assert!(msg.contains("dimension 99"), "got {}", msg);
        }
        ref e => panic!("Expected DimensionMismatch, got {:?}", e),
    }
}
