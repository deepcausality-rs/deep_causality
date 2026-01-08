/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{CWComplex, Cell, CellComplex};

// Test cell implementation with proper boundary
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum SimpleCell {
    Vertex(usize),
    Edge(usize, usize),        // connects vertices
    Face(usize, usize, usize), // triangle
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
                // Boundary: b - a
                vec![(SimpleCell::Vertex(*b), 1), (SimpleCell::Vertex(*a), -1)]
            }
            SimpleCell::Face(a, b, c) => {
                // Boundary: edges (b,c), (a,c), (a,b) with orientations
                vec![
                    (SimpleCell::Edge(*b, *c), 1),
                    (SimpleCell::Edge(*a, *c), -1),
                    (SimpleCell::Edge(*a, *b), 1),
                ]
            }
        }
    }
}

// ============================================================================
// CellComplex construction and getters
// ============================================================================

#[test]
fn test_cell_complex_from_cells() {
    let cells = vec![
        SimpleCell::Vertex(0),
        SimpleCell::Vertex(1),
        SimpleCell::Vertex(2),
        SimpleCell::Edge(0, 1),
        SimpleCell::Edge(1, 2),
        SimpleCell::Edge(0, 2),
        SimpleCell::Face(0, 1, 2),
    ];

    let complex = CellComplex::from_cells(cells);

    assert_eq!(complex.num_cells(0), 3); // 3 vertices
    assert_eq!(complex.num_cells(1), 3); // 3 edges
    assert_eq!(complex.num_cells(2), 1); // 1 face
    assert_eq!(complex.dimension(), 2);
}

#[test]
fn test_cell_complex_cells_vec() {
    let cells = vec![
        SimpleCell::Vertex(0),
        SimpleCell::Vertex(1),
        SimpleCell::Edge(0, 1),
    ];

    let complex = CellComplex::from_cells(cells);

    let vertices = complex.cells_vec(0);
    assert_eq!(vertices.len(), 2);

    let edges = complex.cells_vec(1);
    assert_eq!(edges.len(), 1);

    // Out of bounds returns empty slice
    let no_faces = complex.cells_vec(2);
    assert!(no_faces.is_empty());
}

// ============================================================================
// CWComplex trait implementation
// ============================================================================

#[test]
fn test_cell_complex_cw_cells_iterator() {
    let cells = vec![
        SimpleCell::Vertex(0),
        SimpleCell::Vertex(1),
        SimpleCell::Vertex(2),
    ];

    let complex = CellComplex::from_cells(cells);

    let vertices: Vec<_> = CWComplex::cells(&complex, 0).collect();
    assert_eq!(vertices.len(), 3);
}

#[test]
fn test_cell_complex_max_dim() {
    let cells = vec![SimpleCell::Vertex(0), SimpleCell::Edge(0, 1)];

    let complex = CellComplex::from_cells(cells.clone());
    assert_eq!(CWComplex::max_dim(&complex), 1);

    // Just vertices
    let vertex_only = vec![SimpleCell::Vertex(0)];
    let complex2 = CellComplex::from_cells(vertex_only);
    assert_eq!(CWComplex::max_dim(&complex2), 0);
}

// ============================================================================
// Boundary matrix tests
// ============================================================================

#[test]
fn test_cell_complex_boundary_matrix_k0() {
    // Boundary of 0-cells is empty
    let cells = vec![SimpleCell::Vertex(0), SimpleCell::Vertex(1)];

    let complex = CellComplex::from_cells(cells);
    let boundary = complex.compute_boundary_matrix(0);

    // Should be empty matrix
    assert!(boundary.values().is_empty());
}

#[test]
fn test_cell_complex_boundary_matrix_triangle() {
    // Create a triangle
    let cells = vec![
        SimpleCell::Vertex(0),
        SimpleCell::Vertex(1),
        SimpleCell::Vertex(2),
        SimpleCell::Edge(0, 1),
        SimpleCell::Edge(0, 2),
        SimpleCell::Edge(1, 2),
        SimpleCell::Face(0, 1, 2),
    ];

    let complex = CellComplex::from_cells(cells);

    // ∂_1: edges -> vertices
    let boundary_1 = complex.compute_boundary_matrix(1);
    let (rows, cols) = boundary_1.shape();
    assert_eq!(rows, 3); // 3 vertices
    assert_eq!(cols, 3); // 3 edges
    // Each edge should have exactly 2 non-zero entries (+1, -1)

    // ∂_2: face -> edges
    let boundary_2 = complex.compute_boundary_matrix(2);
    let (rows, cols) = boundary_2.shape();
    assert_eq!(rows, 3); // 3 edges
    assert_eq!(cols, 1); // 1 face
}

// ============================================================================
// Betti number tests
// ============================================================================

#[test]
fn test_cell_complex_betti_number_single_vertex() {
    // Single vertex: b0 = 1 (1 connected component)
    let cells = vec![SimpleCell::Vertex(0)];
    let complex = CellComplex::from_cells(cells);

    let b0 = CWComplex::betti_number(&complex, 0);
    assert_eq!(b0, 1);
}

#[test]
fn test_cell_complex_betti_number_two_vertices() {
    // Two disconnected vertices: b0 = 2
    let cells = vec![SimpleCell::Vertex(0), SimpleCell::Vertex(1)];
    let complex = CellComplex::from_cells(cells);

    let b0 = CWComplex::betti_number(&complex, 0);
    assert_eq!(b0, 2);
}

#[test]
fn test_cell_complex_betti_number_connected_edge() {
    // Two vertices connected by edge: b0 = 1
    let cells = vec![
        SimpleCell::Vertex(0),
        SimpleCell::Vertex(1),
        SimpleCell::Edge(0, 1),
    ];
    let complex = CellComplex::from_cells(cells);

    let b0 = CWComplex::betti_number(&complex, 0);
    assert_eq!(b0, 1);
}

#[test]
fn test_cell_complex_betti_number_filled_triangle() {
    // Filled triangle: b0 = 1 (connected), b1 = 0 (no holes, face fills it)
    let cells = vec![
        SimpleCell::Vertex(0),
        SimpleCell::Vertex(1),
        SimpleCell::Vertex(2),
        SimpleCell::Edge(0, 1),
        SimpleCell::Edge(0, 2),
        SimpleCell::Edge(1, 2),
        SimpleCell::Face(0, 1, 2),
    ];
    let complex = CellComplex::from_cells(cells);

    let b0 = CWComplex::betti_number(&complex, 0);
    assert_eq!(b0, 1);

    let b1 = CWComplex::betti_number(&complex, 1);
    assert_eq!(b1, 0);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_cell_complex_empty() {
    let cells: Vec<SimpleCell> = vec![];
    let complex = CellComplex::from_cells(cells);

    assert_eq!(complex.dimension(), 0);
    assert_eq!(complex.num_cells(0), 0);
}

#[test]
fn test_cell_complex_deduplication() {
    // Same cell added twice should be deduplicated
    let cells = vec![
        SimpleCell::Vertex(0),
        SimpleCell::Vertex(0), // duplicate
        SimpleCell::Vertex(1),
    ];
    let complex = CellComplex::from_cells(cells);

    assert_eq!(complex.num_cells(0), 2); // Only 2 unique vertices
}
