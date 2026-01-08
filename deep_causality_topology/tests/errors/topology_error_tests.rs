/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::CausalMultiVectorError;
use deep_causality_tensor::CausalTensorError;
use deep_causality_topology::{TopologyError, TopologyErrorEnum};

#[test]
fn test_topology_error_simplex_not_found() {
    let err = TopologyError::SimplexNotFound();
    assert_eq!(format!("{}", err), "Simplex not found");
    assert_eq!(err, TopologyError::SimplexNotFound());
}

#[test]
fn test_topology_error_dimension_mismatch() {
    let msg = "Dimensions 3 and 2 are incompatible".to_string();
    let err = TopologyError::DimensionMismatch(msg.clone());
    assert_eq!(format!("{}", err), format!("Dimension mismatch: {}", msg));
    assert_eq!(err, TopologyError::DimensionMismatch(msg));
}

#[test]
fn test_topology_error_invalid_grade_operation() {
    let msg = "Cannot take boundary of 0-chain".to_string();
    let err = TopologyError::InvalidGradeOperation(msg.clone());
    assert_eq!(
        format!("{}", err),
        format!("Invalid grade operation: {}", msg)
    );
    assert_eq!(err, TopologyError::InvalidGradeOperation(msg));
}

#[test]
fn test_topology_error_index_out_of_bounds() {
    let msg = "Index 10 out of bounds for size 5".to_string();
    let err = TopologyError::IndexOutOfBounds(msg.clone());
    assert_eq!(format!("{}", err), format!("Index out of bounds: {}", msg));
    assert_eq!(err, TopologyError::IndexOutOfBounds(msg));
}

#[test]
fn test_topology_error_tensor_error() {
    let msg = "Tensor operation failed".to_string();
    let err = TopologyError::TensorError(msg.clone());
    assert_eq!(format!("{}", err), format!("Tensor error: {}", msg));
    assert_eq!(err, TopologyError::TensorError(msg));
}

#[test]
fn test_topology_error_point_cloud_error() {
    let msg = "PointCloud operation failed".to_string();
    let err = TopologyError::PointCloudError(msg.clone());
    assert_eq!(format!("{}", err), format!("PointCloud error: {}", msg));
    assert_eq!(err, TopologyError::PointCloudError(msg));
}

#[test]
fn test_topology_error_graph_error() {
    let msg = "Graph operation failed".to_string();
    let err = TopologyError::GraphError(msg.clone());
    assert_eq!(format!("{}", err), format!("Graph error: {}", msg));
    assert_eq!(err, TopologyError::GraphError(msg));
}

#[test]
fn test_topology_error_hypergraph_error() {
    let msg = "Hypergraph operation failed".to_string();
    let err = TopologyError::HypergraphError(msg.clone());
    assert_eq!(format!("{}", err), format!("Hypergraph error: {}", msg));
    assert_eq!(err, TopologyError::HypergraphError(msg));
}

#[test]
fn test_topology_error_manifold_error() {
    let msg = "Manifold property not satisfied".to_string();
    let err = TopologyError::ManifoldError(msg.clone());
    assert_eq!(format!("{}", err), format!("Manifold error: {}", msg));
    assert_eq!(err, TopologyError::ManifoldError(msg));
}

#[test]
fn test_topology_error_multivector_error() {
    let msg = "Multivector operation failed".to_string();
    let err = TopologyError::MultivectorError(msg.clone());
    assert_eq!(format!("{}", err), format!("Multivector error: {}", msg));
    assert_eq!(err, TopologyError::MultivectorError(msg));
}

#[test]
fn test_topology_error_invalid_input() {
    let msg = "Invalid input data".to_string();
    let err = TopologyError::InvalidInput(msg.clone());
    assert_eq!(format!("{}", err), format!("Invalid input: {}", msg));
    assert_eq!(err, TopologyError::InvalidInput(msg));
}

#[test]
fn test_topology_error_generic_error() {
    let msg = "Something unexpected happened".to_string();
    let err = TopologyError::GenericError(msg.clone());
    assert_eq!(format!("{}", err), format!("Topology error: {}", msg));
    assert_eq!(err, TopologyError::GenericError(msg));
}

#[test]
fn test_topology_error_from_causal_tensor_error() {
    let tensor_err = CausalTensorError::InvalidParameter("Bad shape".to_string());
    let topology_err: TopologyError = tensor_err.into();
    assert_eq!(
        topology_err,
        TopologyError::TensorError("CausalTensorError: Invalid parameter: Bad shape".to_string())
    );
}

#[test]
fn test_topology_error_from_causal_multivector_error() {
    let multivector_err = CausalMultiVectorError::dimension_mismatch(3, 2);
    let topology_err: TopologyError = multivector_err.into();
    assert_eq!(
        topology_err,
        TopologyError::MultivectorError("Dimension mismatch: expected 3, found 2".to_string())
    );
}

#[test]
fn test_topology_error_invalid_metric() {
    let msg = "Metric tensor is not positive definite".to_string();
    let err = TopologyError::InvalidMetric(msg.clone());
    assert_eq!(format!("{}", err), format!("Invalid metric: {}", msg));
    assert_eq!(err, TopologyError::InvalidMetric(msg));
}

#[test]
fn test_topology_error_gauge_field_error() {
    let msg = "Connection shape mismatch".to_string();
    let err = TopologyError::GaugeFieldError(msg.clone());
    assert_eq!(format!("{}", err), format!("GaugeField error: {}", msg));
    assert_eq!(err, TopologyError::GaugeFieldError(msg));
}

#[test]
fn test_topology_error_new() {
    let err = TopologyError::new(TopologyErrorEnum::SimplexNotFound);
    assert_eq!(format!("{}", err), "Simplex not found");

    let err2 = TopologyError::new(TopologyErrorEnum::GenericError("test".to_string()));
    assert_eq!(format!("{}", err2), "Topology error: test");
}
