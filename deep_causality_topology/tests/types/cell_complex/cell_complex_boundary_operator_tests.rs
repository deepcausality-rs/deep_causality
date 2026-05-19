/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `BoundaryOperator<C>` in `src/types/cell_complex/boundary_operator.rs`.

use deep_causality_topology::{BoundaryOperator, Cell, CellComplex};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum SimpleCell {
    Vertex(usize),
    Edge(usize, usize),
    Face(usize, usize, usize),
}

impl Cell for SimpleCell {
    fn dim(&self) -> usize {
        match self {
            SimpleCell::Vertex(_) => 0,
            SimpleCell::Edge(_, _) => 1,
            SimpleCell::Face(_, _, _) => 2,
        }
    }

    fn boundary(&self) -> Vec<(Self, i8)> {
        match self {
            SimpleCell::Vertex(_) => vec![],
            SimpleCell::Edge(a, b) => {
                vec![(SimpleCell::Vertex(*b), 1), (SimpleCell::Vertex(*a), -1)]
            }
            SimpleCell::Face(a, b, c) => vec![
                (SimpleCell::Edge(*b, *c), 1),
                (SimpleCell::Edge(*a, *c), -1),
                (SimpleCell::Edge(*a, *b), 1),
            ],
        }
    }
}

fn triangle_complex() -> Arc<CellComplex<SimpleCell>> {
    let cells = vec![
        SimpleCell::Vertex(0),
        SimpleCell::Vertex(1),
        SimpleCell::Vertex(2),
        SimpleCell::Edge(0, 1),
        SimpleCell::Edge(0, 2),
        SimpleCell::Edge(1, 2),
        SimpleCell::Face(0, 1, 2),
    ];
    Arc::new(CellComplex::from_cells(cells))
}

#[test]
fn test_boundary_operator_dims_for_k1() {
    let complex = triangle_complex();
    let op = BoundaryOperator::new(complex, 1);
    assert_eq!(op.source_dim(), 1);
    assert_eq!(op.target_dim(), 0);
}

#[test]
fn test_boundary_operator_target_dim_k0_saturates() {
    let complex = triangle_complex();
    let op = BoundaryOperator::new(complex, 0);
    // k = 0 -> target should saturate to 0, not underflow.
    assert_eq!(op.source_dim(), 0);
    assert_eq!(op.target_dim(), 0);
}

#[test]
fn test_boundary_operator_matrix_shape_k1() {
    let complex = triangle_complex();
    let op = BoundaryOperator::new(complex, 1);
    // 3 vertices x 3 edges
    let (rows, cols) = op.matrix().shape();
    assert_eq!(rows, 3);
    assert_eq!(cols, 3);
}

#[test]
fn test_boundary_operator_apply_single_edge() {
    let complex = triangle_complex();
    let op = BoundaryOperator::new(Arc::clone(&complex), 1);

    // Apply boundary to the chain [Edge(0,1)] with coeff +1.
    // ∂(0,1) = Vertex(1) - Vertex(0)
    let chain = vec![(SimpleCell::Edge(0, 1), 1i8)];
    let result = op.apply(&chain);

    // Result should be a chain of vertices summing to {V1: +1, V0: -1}.
    let mut v0_coeff = 0i8;
    let mut v1_coeff = 0i8;
    for (cell, coeff) in result {
        match cell {
            SimpleCell::Vertex(0) => v0_coeff = coeff,
            SimpleCell::Vertex(1) => v1_coeff = coeff,
            _ => {}
        }
    }
    assert_eq!(v0_coeff, -1);
    assert_eq!(v1_coeff, 1);
}

#[test]
fn test_boundary_operator_apply_face_to_edges() {
    let complex = triangle_complex();
    let op = BoundaryOperator::new(Arc::clone(&complex), 2);

    let chain = vec![(SimpleCell::Face(0, 1, 2), 1i8)];
    let result = op.apply(&chain);

    // Boundary should produce a non-empty result of edges.
    assert!(!result.is_empty());

    // Sum of absolute coefficients matches non-zero entries in ∂_2 column.
    let total: i8 = result.iter().map(|(_, c)| c.abs()).sum();
    assert!(total >= 1);
}

#[test]
fn test_boundary_operator_apply_empty_chain() {
    let complex = triangle_complex();
    let op = BoundaryOperator::new(complex, 1);

    let chain: Vec<(SimpleCell, i8)> = Vec::new();
    let result = op.apply(&chain);
    assert!(
        result.is_empty(),
        "Applying boundary to an empty chain should be empty"
    );
}

#[test]
fn test_boundary_operator_apply_unknown_cell_is_ignored() {
    let complex = triangle_complex();
    let op = BoundaryOperator::new(complex, 1);

    // An edge that does not belong to the complex must be silently ignored
    // by the index map lookup, not panic.
    let chain = vec![(SimpleCell::Edge(7, 9), 1i8)];
    let result = op.apply(&chain);
    assert!(result.is_empty());
}

#[test]
fn test_boundary_operator_apply_squared_is_zero() {
    // ∂_{k-1} ∘ ∂_k = 0. Apply ∂_2 then ∂_1; result must be empty/zero.
    let complex = triangle_complex();
    let op2 = BoundaryOperator::new(Arc::clone(&complex), 2);
    let op1 = BoundaryOperator::new(Arc::clone(&complex), 1);

    let chain = vec![(SimpleCell::Face(0, 1, 2), 1i8)];
    let edges = op2.apply(&chain);
    let vertices = op1.apply(&edges);

    // The composed boundary should be zero, which is represented as an empty chain
    // (the implementation skips zero coefficients).
    assert!(vertices.is_empty(), "∂² should be zero, got {:?}", vertices);
}
