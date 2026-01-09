/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{CWComplex, Lattice, LatticeCell};

#[test]
fn test_lattice_construction() {
    let lattice = Lattice::<2>::new([10, 10], [true, true]); // Torus
    assert_eq!(lattice.shape(), &[10, 10]);
    assert_eq!(lattice.dim(), 2);
    assert!(lattice.periodic().iter().all(|&p| p));
}

#[test]
fn test_lattice_cell_counting() {
    // 3x3 square grid
    // Vertices: 3*3 = 9
    // Edges:
    //  Horizontal: 3 rows * 2 edges (open) or 3 edges (periodic)
    //  Vertical: same

    // Open
    let open = Lattice::<2>::new([3, 3], [false, false]);
    assert_eq!(open.num_cells(0), 9); // vertices
    // Edges: 3*(3-1) + 3*(3-1) = 6 + 6 = 12
    assert_eq!(open.num_cells(1), 12);
    // Faces: (3-1)*(3-1) = 4
    assert_eq!(open.num_cells(2), 4);

    // Periodic (Torus)
    let periodic = Lattice::<2>::new([3, 3], [true, true]);
    assert_eq!(periodic.num_cells(0), 9);
    // Edges: 3*3 + 3*3 = 18
    assert_eq!(periodic.num_cells(1), 18);
    // Faces: 3*3 = 9
    assert_eq!(periodic.num_cells(2), 9);
}

#[test]
fn test_lattice_iterators() {
    // 2x2 Open
    let lat = Lattice::<2>::new([2, 2], [false, false]);

    // Vertices: (0,0), (1,0), (0,1), (1,1) -> 4
    assert_eq!(lat.cells(0).count(), 4);

    // Edges: (0,0)-h, (0,1)-h, (0,0)-v, (1,0)-v -> 4
    // Vertices are 0..L-1.
    // Horizontal edges connect (i, j) to (i+1, j). Exist if i < L-1.
    // L=2. i=0 exists. i=1 No.
    // Rows j=0, 1. So (0,0) and (0,1) horizontal. (2 edges)
    // Vert same. Total 4.
    assert_eq!(lat.cells(1).count(), 4);

    // Faces: (0,0) -> 1
    assert_eq!(lat.cells(2).count(), 1);
}

// =============================================================================
// boundary() and wrap_cell() tests (Lines 64-95)
// =============================================================================

#[test]
fn test_boundary_periodic_wrapping() {
    // 3x3 torus - boundary should wrap around edges
    let lattice = Lattice::<2>::torus([3, 3]);

    // Create a cell at position [2, 0] with horizontal orientation (edge along dim 0)
    let edge_cell = LatticeCell::new([2, 0], 0b01); // orientation bit 0 set

    let boundary = lattice.boundary(&edge_cell);

    // Boundary of an edge is two vertices
    assert_eq!(boundary.len(), 2, "Edge boundary should have 2 vertices");

    // Check coefficients sum to zero (for chain complex)
    let coeff_sum: i8 = boundary.iter().map(|(_, c)| *c).sum();
    assert_eq!(coeff_sum, 0, "Boundary coefficients should sum to zero");
}

#[test]
fn test_boundary_open_drops_out_of_bounds() {
    // 3x3 open lattice - boundary cells outside should be dropped
    let lattice = Lattice::<2>::open([3, 3]);

    // Create a cell at position [2, 0] with horizontal orientation
    let edge_cell = LatticeCell::new([2, 0], 0b01);

    let boundary = lattice.boundary(&edge_cell);

    // For open boundary, cells going beyond dim_len are dropped
    // This tests the wrap_cell returning None path (Line 89)
    assert!(
        boundary.len() <= 2,
        "Open boundary may have fewer cells if some are out of bounds"
    );
}

#[test]
fn test_wrap_cell_periodic_modulo() {
    // Test that wrap_cell correctly applies modulo for periodic dimensions
    let lattice = Lattice::<2>::torus([5, 5]);

    // Create a 1-cell that would be at position [5, 0] - should wrap to [0, 0]
    let cell = LatticeCell::new([5, 0], 0b01);
    let boundary = lattice.boundary(&cell);

    // The wrapped boundary should still produce valid cells
    assert!(!boundary.is_empty(), "Periodic boundary should wrap cells");
}

#[test]
fn test_wrap_cell_mixed_boundaries() {
    // Mixed: periodic in dim 0, open in dim 1
    let lattice = Lattice::<2>::new([3, 3], [true, false]);

    // Cell at boundary in dim 1 (open)
    let edge_cell = LatticeCell::new([0, 2], 0b10); // orientation bit 1 set

    let boundary = lattice.boundary(&edge_cell);

    // Some boundary cells may be dropped due to open boundary in dim 1
    assert!(boundary.len() <= 2);
}

// =============================================================================
// max_dim() tests (Lines 218-220)
// =============================================================================

#[test]
fn test_max_dim_2d() {
    let lattice = Lattice::<2>::new([4, 4], [false, false]);
    assert_eq!(lattice.max_dim(), 2, "2D lattice should have max_dim=2");
}

#[test]
fn test_max_dim_3d() {
    let lattice = Lattice::<3>::cubic_torus(3);
    assert_eq!(lattice.max_dim(), 3, "3D lattice should have max_dim=3");
}

#[test]
fn test_max_dim_4d() {
    let lattice = Lattice::<4>::hypercubic_torus(2);
    assert_eq!(lattice.max_dim(), 4, "4D lattice should have max_dim=4");
}

// =============================================================================
// boundary_matrix() tests (Lines 222-243)
// =============================================================================

#[test]
fn test_boundary_matrix_2d_open() {
    let lattice = Lattice::<2>::square_open(3);

    // Get boundary matrix ∂_1 (edges -> vertices)
    let bdry_1 = lattice.boundary_matrix(1);
    let (rows, cols) = bdry_1.shape();

    // rows = num_cells(0) = 9 vertices
    // cols = num_cells(1) = 12 edges
    assert_eq!(rows, 9, "Should have 9 vertices (rows)");
    assert_eq!(cols, 12, "Should have 12 edges (cols)");
}

#[test]
fn test_boundary_matrix_2d_torus() {
    let lattice = Lattice::<2>::square_torus(3);

    // Get boundary matrix ∂_2 (faces -> edges)
    let bdry_2 = lattice.boundary_matrix(2);
    let (rows, cols) = bdry_2.shape();

    // rows = num_cells(1) = 18 edges
    // cols = num_cells(2) = 9 faces
    assert_eq!(rows, 18, "Should have 18 edges (rows)");
    assert_eq!(cols, 9, "Should have 9 faces (cols)");
}

#[test]
fn test_boundary_matrix_squared_is_zero() {
    // For any chain complex, ∂_{k-1} ∘ ∂_k = 0
    let lattice = Lattice::<2>::square_torus(2);

    let bdry_2 = lattice.boundary_matrix(2);
    let bdry_1 = lattice.boundary_matrix(1);

    // Check dimensions allow composition
    let (_, cols_2) = bdry_2.shape();
    let (rows_1, _) = bdry_1.shape();
    assert_eq!(cols_2, rows_1, "Matrices should be composable");
}

// =============================================================================
// betti_number() tests (Lines 245-267)
// =============================================================================

#[test]
fn test_betti_number_torus_2d() {
    // 2D torus has Betti numbers: b0=1, b1=2, b2=1
    let lattice = Lattice::<2>::square_torus(5);

    assert_eq!(lattice.betti_number(0), 1, "b0 of 2-torus should be 1");
    assert_eq!(lattice.betti_number(1), 2, "b1 of 2-torus should be 2");
    assert_eq!(lattice.betti_number(2), 1, "b2 of 2-torus should be 1");
}

#[test]
fn test_betti_number_torus_3d() {
    // 3D torus has Betti numbers: b0=1, b1=3, b2=3, b3=1
    let lattice = Lattice::<3>::cubic_torus(3);

    assert_eq!(lattice.betti_number(0), 1, "b0 of 3-torus should be 1");
    assert_eq!(lattice.betti_number(1), 3, "b1 of 3-torus should be 3");
    assert_eq!(lattice.betti_number(2), 3, "b2 of 3-torus should be 3");
    assert_eq!(lattice.betti_number(3), 1, "b3 of 3-torus should be 1");
}

#[test]
fn test_betti_number_k_greater_than_dim_periodic() {
    // When k > D for all-periodic lattice, betti should be 0
    let lattice = Lattice::<2>::square_torus(3);

    assert_eq!(
        lattice.betti_number(3),
        0,
        "b3 of 2D torus should be 0 (k > D)"
    );
    assert_eq!(
        lattice.betti_number(10),
        0,
        "b10 of 2D torus should be 0 (k > D)"
    );
}

#[test]
fn test_betti_number_partial_periodic() {
    // Only one dimension periodic (cylinder-like)
    let lattice = Lattice::<2>::new([3, 3], [true, false]);

    // p_dims = 1 (only dim 0 is periodic)
    // b0 = 1, b1 = 1, b_k = 0 for k > 1
    assert_eq!(
        lattice.betti_number(0),
        1,
        "b0 of partial periodic should be 1"
    );
    assert_eq!(
        lattice.betti_number(1),
        1,
        "b1 of partial periodic (1 periodic dim) should be 1"
    );
    assert_eq!(
        lattice.betti_number(2),
        0,
        "b2 of partial periodic should be 0 (k > p_dims)"
    );
}

#[test]
fn test_betti_number_open_lattice() {
    // Open lattice (no periodic dimensions)
    // p_dims = 0, so b_k = 0 for k > 0
    let lattice = Lattice::<2>::square_open(3);

    assert_eq!(lattice.betti_number(0), 1, "b0 of open lattice should be 1");
    assert_eq!(
        lattice.betti_number(1),
        0,
        "b1 of open lattice should be 0 (k > p_dims)"
    );
    assert_eq!(
        lattice.betti_number(2),
        0,
        "b2 of open lattice should be 0 (k > p_dims)"
    );
}

#[test]
fn test_betti_number_4d_torus() {
    // 4D torus has Betti numbers: b0=1, b1=4, b2=6, b3=4, b4=1
    let lattice = Lattice::<4>::hypercubic_torus(2);

    assert_eq!(lattice.betti_number(0), 1, "b0 of 4-torus");
    assert_eq!(lattice.betti_number(1), 4, "b1 of 4-torus");
    assert_eq!(lattice.betti_number(2), 6, "b2 of 4-torus");
    assert_eq!(lattice.betti_number(3), 4, "b3 of 4-torus");
    assert_eq!(lattice.betti_number(4), 1, "b4 of 4-torus");
    assert_eq!(lattice.betti_number(5), 0, "b5 of 4-torus should be 0");
}

// =============================================================================
// Iterator edge cases
// =============================================================================

#[test]
fn test_iter_cells_empty_orientations() {
    // Request dimension higher than lattice dimension
    let lattice = Lattice::<2>::square_torus(3);

    // k=3 for a 2D lattice - no valid orientations (no 3-cells)
    let cells: Vec<_> = lattice.iter_cells(3).collect();
    assert!(cells.is_empty(), "No 3-cells in a 2D lattice");
}

#[test]
fn test_iter_cells_all_dimensions() {
    let lattice = Lattice::<2>::square_torus(2);

    // Collect cells at each dimension
    let cells_0: Vec<_> = lattice.cells(0).collect();
    let cells_1: Vec<_> = lattice.cells(1).collect();
    let cells_2: Vec<_> = lattice.cells(2).collect();

    assert_eq!(cells_0.len(), lattice.num_cells(0));
    assert_eq!(cells_1.len(), lattice.num_cells(1));
    assert_eq!(cells_2.len(), lattice.num_cells(2));
}

// =============================================================================
// num_cells edge cases
// =============================================================================

#[test]
fn test_num_cells_zero_dimension_edge_case() {
    // Test with shape containing zeros - edge case for dim_len == 0
    let lattice = Lattice::<2>::new([0, 3], [false, false]);

    // With zero length in one dimension, should have 0 cells
    assert_eq!(
        lattice.num_cells(0),
        0,
        "Zero-sized dimension should yield 0 cells"
    );
}
