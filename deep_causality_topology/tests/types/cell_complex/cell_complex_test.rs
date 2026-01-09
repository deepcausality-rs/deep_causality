/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::Cell;
use deep_causality_topology::{CWComplex, CellComplex};

/// A mock cell with proper boundary implementation for testing
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct TestCell {
    id: usize,
    dim: usize,
    boundary_cells: Vec<(usize, usize, i8)>, // (id, dim, coeff)
}

impl TestCell {
    fn vertex(id: usize) -> Self {
        Self {
            id,
            dim: 0,
            boundary_cells: Vec::new(),
        }
    }

    fn edge(id: usize, v0: usize, v1: usize) -> Self {
        Self {
            id,
            dim: 1,
            boundary_cells: vec![(v0, 0, -1), (v1, 0, 1)],
        }
    }

    fn face(id: usize, edges: Vec<(usize, i8)>) -> Self {
        Self {
            id,
            dim: 2,
            boundary_cells: edges.into_iter().map(|(e, c)| (e, 1, c)).collect(),
        }
    }
}

impl Cell for TestCell {
    fn dim(&self) -> usize {
        self.dim
    }

    fn boundary(&self) -> Vec<(Self, i8)> {
        self.boundary_cells
            .iter()
            .map(|(id, dim, coeff)| {
                (
                    TestCell {
                        id: *id,
                        dim: *dim,
                        boundary_cells: Vec::new(),
                    },
                    *coeff,
                )
            })
            .collect()
    }
}

// =============================================================================
// cells_vec() edge cases
// =============================================================================

#[test]
fn test_cells_vec_out_of_bounds() {
    let cells = vec![TestCell::vertex(0), TestCell::vertex(1)];

    let complex = CellComplex::from_cells(cells);

    // Requesting dimension higher than exists should return empty slice
    let result = complex.cells_vec(5);
    assert!(
        result.is_empty(),
        "Out-of-bounds dimension should return empty slice"
    );
}

#[test]
fn test_cells_vec_valid_dimensions() {
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::edge(2, 0, 1),
    ];

    let complex = CellComplex::from_cells(cells);

    assert_eq!(complex.cells_vec(0).len(), 2, "Should have 2 vertices");
    assert_eq!(complex.cells_vec(1).len(), 1, "Should have 1 edge");
}

// =============================================================================
// dimension() tests
// =============================================================================

#[test]
fn test_dimension_empty_complex() {
    let complex = CellComplex::<TestCell>::from_cells(vec![]);
    // Empty complex has dimension 0 (saturating_sub(1) on empty vec)
    // Actually from_cells returns an empty cells vec, so dimension = 0-1 saturating = 0
    assert_eq!(complex.dimension(), 0);
}

// =============================================================================
// compute_boundary_matrix() edge cases
// =============================================================================

#[test]
fn test_compute_boundary_matrix_k_zero() {
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::edge(2, 0, 1),
    ];

    let complex = CellComplex::from_cells(cells);

    // k=0: boundary of 0-cells (vertices) is empty
    let bdry = complex.compute_boundary_matrix(0);
    assert_eq!(bdry.shape(), (0, 0), "∂_0 should be empty matrix");
}

#[test]
fn test_compute_boundary_matrix_k_beyond_dim() {
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::edge(2, 0, 1),
    ];

    let complex = CellComplex::from_cells(cells);

    // k > dimension: should return empty matrix
    let bdry = complex.compute_boundary_matrix(10);
    assert_eq!(bdry.shape(), (0, 0), "∂_k for k > dim should be empty");
}

#[test]
fn test_compute_boundary_matrix_valid() {
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::vertex(2),
        TestCell::edge(3, 0, 1),
        TestCell::edge(4, 1, 2),
        TestCell::edge(5, 2, 0),
    ];

    let complex = CellComplex::from_cells(cells);

    // ∂_1 maps edges to vertices
    let bdry = complex.compute_boundary_matrix(1);
    let (rows, cols) = bdry.shape();

    assert_eq!(rows, 3, "Should have 3 vertices (rows)");
    assert_eq!(cols, 3, "Should have 3 edges (cols)");
}

// =============================================================================
// betti_number() tests (uses rank_of_matrix internally)
// =============================================================================

#[test]
fn test_betti_number_single_vertex() {
    let cells = vec![TestCell::vertex(0)];

    let complex = CellComplex::from_cells(cells);

    // Single vertex: b0 = 1, b_k = 0 for k > 0
    assert_eq!(
        complex.betti_number(0),
        1,
        "b0 of single vertex should be 1"
    );
    assert_eq!(
        complex.betti_number(1),
        0,
        "b1 of single vertex should be 0"
    );
}

#[test]
fn test_betti_number_disconnected_vertices() {
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::vertex(2),
    ];

    let complex = CellComplex::from_cells(cells);

    // 3 disconnected vertices: b0 = 3
    assert_eq!(
        complex.betti_number(0),
        3,
        "b0 of 3 disconnected vertices should be 3"
    );
}

#[test]
fn test_betti_number_connected_graph() {
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::edge(2, 0, 1),
    ];

    let complex = CellComplex::from_cells(cells);

    // Two vertices connected by edge: b0 = 1 (connected), b1 = 0
    assert_eq!(
        complex.betti_number(0),
        1,
        "b0 of connected graph should be 1"
    );
}

#[test]
fn test_betti_number_triangle() {
    // Triangle: 3 vertices, 3 edges, 1 face
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::vertex(2),
        TestCell::edge(3, 0, 1),
        TestCell::edge(4, 1, 2),
        TestCell::edge(5, 2, 0),
        TestCell::face(6, vec![(3, 1), (4, 1), (5, -1)]),
    ];

    let complex = CellComplex::from_cells(cells);

    // Filled triangle (disk): b0=1, b1=0, b2=0
    assert_eq!(complex.betti_number(0), 1, "b0 of filled triangle");
}

// =============================================================================
// CWComplex trait implementation tests
// =============================================================================

#[test]
fn test_cwcomplex_cells_iterator() {
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::edge(2, 0, 1),
    ];

    let complex = CellComplex::from_cells(cells);

    // Test cells() iterator
    let vertices: Vec<_> = complex.cells(0).collect();
    let edges: Vec<_> = complex.cells(1).collect();

    assert_eq!(vertices.len(), 2);
    assert_eq!(edges.len(), 1);
}

#[test]
fn test_cwcomplex_max_dim() {
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::edge(2, 0, 1),
    ];

    let complex = CellComplex::from_cells(cells);

    assert_eq!(complex.max_dim(), 1, "max_dim should be 1 for edge complex");
}

#[test]
fn test_cwcomplex_boundary_matrix() {
    let cells = vec![
        TestCell::vertex(0),
        TestCell::vertex(1),
        TestCell::edge(2, 0, 1),
    ];

    let complex = CellComplex::from_cells(cells);

    // Test via CWComplex trait
    let bdry = complex.boundary_matrix(1);
    let (rows, cols) = bdry.shape();

    assert_eq!(rows, 2, "Should have 2 rows (vertices)");
    assert_eq!(cols, 1, "Should have 1 column (edge)");
}
