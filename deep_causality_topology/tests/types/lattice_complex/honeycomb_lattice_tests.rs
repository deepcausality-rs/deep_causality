/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{ChainComplex, HoneycombLattice};

// ============================================================================
// HoneycombLattice construction
// ============================================================================

#[test]
fn test_honeycomb_lattice_new() {
    let lattice = HoneycombLattice::new([3, 4], [false, false]);

    assert_eq!(lattice.size(), &[3, 4]);
    assert_eq!(lattice.periodic(), &[false, false]);
}

#[test]
fn test_honeycomb_lattice_periodic() {
    let periodic = HoneycombLattice::new([5, 5], [true, true]);

    assert_eq!(periodic.periodic(), &[true, true]);
}

#[test]
fn test_honeycomb_lattice_size() {
    let lattice = HoneycombLattice::new([10, 20], [false, false]);

    assert_eq!(lattice.size()[0], 10);
    assert_eq!(lattice.size()[1], 20);
}

// ============================================================================
// HoneycombLattice as CellComplex
// ============================================================================

#[test]
fn test_honeycomb_as_cell_complex_vertices() {
    // Each unit cell has 2 sites (A and B)
    let lattice = HoneycombLattice::new([2, 2], [false, false]);
    let complex = lattice.as_cell_complex();

    // 2x2 grid = 4 cells, each with 2 vertices = 8 vertices
    assert_eq!(complex.num_cells(0), 8);
}

#[test]
fn test_honeycomb_as_cell_complex_faces() {
    // Each unit cell has 1 hexagon face
    let lattice = HoneycombLattice::new([3, 3], [false, false]);
    let complex = lattice.as_cell_complex();

    // 3x3 grid = 9 hexagons
    assert_eq!(complex.num_cells(2), 9);
}

#[test]
fn test_honeycomb_as_cell_complex_dimension() {
    let lattice = HoneycombLattice::new([2, 2], [false, false]);
    let complex = lattice.as_cell_complex();

    assert_eq!(ChainComplex::max_dim(&complex), 2);
}

#[test]
fn test_honeycomb_as_cell_complex_edges() {
    // For a 2x2 honeycomb:
    // Bond 0 (A-B internal): always exists, 4 edges
    // Bond 1 (left): exists if c > 0, so 2 edges (c=1 for rows 0,1)
    // Bond 2 (top): exists if r > 0, so 2 edges (r=1 for cols 0,1)
    // Total: 4 + 2 + 2 = 8 edges for open boundary
    let lattice = HoneycombLattice::new([2, 2], [false, false]);
    let complex = lattice.as_cell_complex();

    let num_edges = complex.num_cells(1);
    assert!(num_edges > 0, "Should have some edges");
}

#[test]
fn test_honeycomb_boundary_matrix_dim_1() {
    // boundary_matrix(1) iterates over edges and calls Cell::boundary()
    // on each (subtype 0, 1, 2). This exercises the bond subtypes.
    let lattice = HoneycombLattice::new([2, 2], [false, false]);
    let complex = lattice.as_cell_complex();
    let bm = complex.compute_boundary_matrix(1);
    let shape = bm.shape();
    // rows = num vertices (8), cols = num edges (>=4)
    assert_eq!(shape.0, 8);
    assert!(shape.1 > 0);
    // For each edge with subtype 0 (always present), boundary should produce 2 nonzero entries
    // overall there should be nonzero entries
    assert!(!bm.values().is_empty());
}

#[test]
fn test_honeycomb_boundary_matrix_dim_2() {
    // boundary_matrix(2) iterates faces (dim 2). Face boundary is empty placeholder.
    let lattice = HoneycombLattice::new([2, 2], [false, false]);
    let complex = lattice.as_cell_complex();
    let bm = complex.compute_boundary_matrix(2);
    // Should have rows=num edges, cols=num faces, but all empty
    assert!(bm.values().is_empty());
}

#[test]
fn test_honeycomb_boundary_dim_0_empty() {
    // Vertices boundary() returns vec![] — exercise via compute_boundary_matrix(0)
    let lattice = HoneycombLattice::new([1, 1], [false, false]);
    let complex = lattice.as_cell_complex();
    let bm = complex.compute_boundary_matrix(0);
    // boundary of 0-chains is empty
    assert!(bm.values().is_empty());
}

#[test]
fn test_honeycomb_boundary_subtype_1_and_2() {
    // Use a 2x2 lattice so both bond subtype 1 (left) and 2 (top) exist
    // and exercise their boundary branches with (r>0, c>0).
    let lattice = HoneycombLattice::new([3, 3], [false, false]);
    let complex = lattice.as_cell_complex();
    let bm = complex.compute_boundary_matrix(1);
    // Should have many non-zero entries
    assert!(bm.values().len() >= 4);
}

#[test]
fn test_honeycomb_lattice_small() {
    // Minimal 1x1 honeycomb
    let lattice = HoneycombLattice::new([1, 1], [false, false]);
    let complex = lattice.as_cell_complex();

    // 1 unit cell: 2 vertices, 1 internal edge, 1 hexagon
    assert_eq!(complex.num_cells(0), 2);
    assert_eq!(complex.num_cells(2), 1);
}
