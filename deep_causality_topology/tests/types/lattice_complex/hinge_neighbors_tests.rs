/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `LatticeComplex::hinge_top_cube_neighbors` — Phase R2 task 3.2.

use deep_causality_topology::{ChainComplex, LatticeCell, LatticeComplex};

/// Locate the flat `CellId` for a cell matching `target` in `lattice.cells(grade)` order.
fn find_cell_id<const D: usize>(
    lattice: &LatticeComplex<D, f64>,
    grade: usize,
    target: &LatticeCell<D>,
) -> usize {
    lattice
        .cells(grade)
        .position(|c| c == *target)
        .unwrap_or_else(|| panic!("cell {target:?} not found in grade {grade}"))
}

// -- 2D: hinges are vertices (D-2 = 0) ----------------------------------------------

#[test]
fn periodic_2d_interior_vertex_has_4_incident_squares() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let vertex = LatticeCell::vertex([1, 1]);
    let hinge_id = find_cell_id(&lattice, 0, &vertex);
    let neighbors = lattice.hinge_top_cube_neighbors(hinge_id);
    assert_eq!(neighbors.len(), 4, "interior vertex on periodic lattice");
}

#[test]
fn periodic_2d_corner_vertex_also_has_4_incident_squares() {
    // On a torus, every vertex is interior — the boundary wraps. (0,0) still touches 4 squares.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let vertex = LatticeCell::vertex([0, 0]);
    let hinge_id = find_cell_id(&lattice, 0, &vertex);
    let neighbors = lattice.hinge_top_cube_neighbors(hinge_id);
    assert_eq!(neighbors.len(), 4);
}

#[test]
fn open_2d_corner_vertex_has_1_incident_square() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let corner = LatticeCell::vertex([0, 0]);
    let hinge_id = find_cell_id(&lattice, 0, &corner);
    let neighbors = lattice.hinge_top_cube_neighbors(hinge_id);
    assert_eq!(
        neighbors.len(),
        1,
        "open-lattice corner has one incident square"
    );
}

#[test]
fn open_2d_boundary_edge_vertex_has_2_incident_squares() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    // (1, 0) sits on the bottom boundary — should see 2 squares above it.
    let v = LatticeCell::vertex([1, 0]);
    let hinge_id = find_cell_id(&lattice, 0, &v);
    let neighbors = lattice.hinge_top_cube_neighbors(hinge_id);
    assert_eq!(neighbors.len(), 2);
}

#[test]
fn open_2d_interior_vertex_has_4_incident_squares() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let v = LatticeCell::vertex([1, 1]);
    let hinge_id = find_cell_id(&lattice, 0, &v);
    let neighbors = lattice.hinge_top_cube_neighbors(hinge_id);
    assert_eq!(neighbors.len(), 4);
}

#[test]
fn neighbors_are_deduplicated_and_within_range() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let num_top = lattice.num_cells(2);
    for hinge_id in 0..lattice.num_cells(0) {
        let neighbors = lattice.hinge_top_cube_neighbors(hinge_id);
        // Deduplicated:
        let mut sorted = neighbors.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            neighbors.len(),
            "duplicates at hinge {hinge_id}"
        );
        // In range:
        assert!(neighbors.iter().all(|&id| id < num_top));
    }
}

#[test]
fn out_of_range_hinge_id_returns_empty() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let n = lattice.num_cells(0);
    assert!(lattice.hinge_top_cube_neighbors(n).is_empty());
    assert!(lattice.hinge_top_cube_neighbors(n + 100).is_empty());
}

// -- 3D: hinges are edges (D-2 = 1) -------------------------------------------------

#[test]
fn periodic_3d_interior_edge_has_4_incident_cubes() {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(3);
    // An edge along axis 0 starting at (1, 1, 1).
    let edge = LatticeCell::edge([1, 1, 1], 0);
    let hinge_id = find_cell_id(&lattice, 1, &edge);
    let neighbors = lattice.hinge_top_cube_neighbors(hinge_id);
    assert_eq!(neighbors.len(), 4);
}

#[test]
fn open_3d_corner_edge_has_1_incident_cube() {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_open(3);
    // Edge along axis 0 at the corner (0, 0, 0) — only the (0,0,0) cube contains it.
    let edge = LatticeCell::edge([0, 0, 0], 0);
    let hinge_id = find_cell_id(&lattice, 1, &edge);
    let neighbors = lattice.hinge_top_cube_neighbors(hinge_id);
    assert_eq!(neighbors.len(), 1);
}

#[test]
fn periodic_3d_all_edge_hinges_have_4_cubes() {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(3);
    for hinge_id in 0..lattice.num_cells(1) {
        let n = lattice.hinge_top_cube_neighbors(hinge_id);
        assert_eq!(n.len(), 4, "hinge {hinge_id} on torus should see 4 cubes");
    }
}

// -- 4D: hinges are 2-cells (D-2 = 2) -----------------------------------------------

#[test]
fn periodic_4d_interior_2cell_has_4_incident_4cubes() {
    let lattice: LatticeComplex<4, f64> = LatticeComplex::hypercubic_torus(2);
    // Walk a few 2-cell hinges and verify each has exactly 4 incident 4-cubes.
    let max_check = 8.min(lattice.num_cells(2));
    for hinge_id in 0..max_check {
        let neighbors = lattice.hinge_top_cube_neighbors(hinge_id);
        assert_eq!(
            neighbors.len(),
            4,
            "4D torus hinge {hinge_id} should see 4 4-cubes, got {}",
            neighbors.len(),
        );
    }
}
