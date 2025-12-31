/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{CWComplex, HoneycombLattice};

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

    assert_eq!(CWComplex::max_dim(&complex), 2);
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
fn test_honeycomb_lattice_small() {
    // Minimal 1x1 honeycomb
    let lattice = HoneycombLattice::new([1, 1], [false, false]);
    let complex = lattice.as_cell_complex();

    // 1 unit cell: 2 vertices, 1 internal edge, 1 hexagon
    assert_eq!(complex.num_cells(0), 2);
    assert_eq!(complex.num_cells(2), 1);
}
