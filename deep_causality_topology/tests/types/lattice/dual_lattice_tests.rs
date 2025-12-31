/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{DualLattice, Lattice, LatticeCell};
use std::sync::Arc;

// ============================================================================
// DualLattice Constructors
// ============================================================================

#[test]
fn test_dual_lattice_new() {
    let primal = Lattice::<2>::new([4, 4], [true, true]);
    let dual = DualLattice::new(primal);

    assert_eq!(dual.primal().shape(), &[4, 4]);
    assert_eq!(dual.primal().periodic(), &[true, true]);
}

#[test]
fn test_dual_lattice_new_arc() {
    let primal = Arc::new(Lattice::<3>::cubic_torus(3));
    let dual = DualLattice::new_arc(Arc::clone(&primal));

    assert_eq!(dual.primal().shape(), &[3, 3, 3]);
}

// ============================================================================
// DualLattice Duality Operations
// ============================================================================

#[test]
fn test_dual_cell_vertex_to_volume() {
    // In 2D: dual of vertex (0-cell) is a face (2-cell)
    let primal = Lattice::<2>::square_torus(4);
    let dual = DualLattice::new(primal);

    let vertex = LatticeCell::<2>::vertex([1, 1]);
    let dual_cell = dual.dual_cell(&vertex);

    // Dual of 0-cell is (D-0)-cell = 2-cell
    assert_eq!(dual_cell.cell_dim(), 2);
}

#[test]
fn test_dual_cell_edge_to_edge() {
    // In 2D: dual of edge (1-cell) is an edge (1-cell)
    let primal = Lattice::<2>::square_torus(4);
    let dual = DualLattice::new(primal);

    let edge_x = LatticeCell::<2>::edge([1, 1], 0); // x-direction
    let dual_edge = dual.dual_cell(&edge_x);

    // Dual of 1-cell in 2D is (2-1)-cell = 1-cell
    assert_eq!(dual_edge.cell_dim(), 1);
    // The orientation should be perpendicular (complement)
    assert_eq!(dual_edge.orientation(), 0b10); // y-direction
}

#[test]
fn test_dual_cell_face_to_vertex() {
    // In 2D: dual of face (2-cell) is a vertex (0-cell)
    let primal = Lattice::<2>::square_torus(4);
    let dual = DualLattice::new(primal);

    let face = LatticeCell::<2>::new([0, 0], 0b11);
    let dual_cell = dual.dual_cell(&face);

    // Dual of 2-cell is (2-2)-cell = 0-cell
    assert_eq!(dual_cell.cell_dim(), 0);
}

#[test]
fn test_dual_cell_3d_edge_to_face() {
    // In 3D: dual of edge (1-cell) is a face (2-cell)
    let primal = Lattice::<3>::cubic_torus(3);
    let dual = DualLattice::new(primal);

    let edge_z = LatticeCell::<3>::edge([0, 0, 0], 2); // z-direction (0b100)
    let dual_cell = dual.dual_cell(&edge_z);

    // Dual of 1-cell in 3D is (3-1)-cell = 2-cell
    assert_eq!(dual_cell.cell_dim(), 2);
    // Orientation complement: !0b100 & 0b111 = 0b011 (xy-plane)
    assert_eq!(dual_cell.orientation(), 0b011);
}

#[test]
fn test_dual_dual_equals_original() {
    // Poincaré duality: dual(dual(cell)) = cell
    let primal = Lattice::<2>::square_torus(4);
    let dual = DualLattice::new(primal);

    let vertex = LatticeCell::<2>::vertex([2, 3]);
    let double_dual = dual.dual_cell(&dual.dual_cell(&vertex));

    assert_eq!(double_dual.position(), vertex.position());
    assert_eq!(double_dual.orientation(), vertex.orientation());
}

// ============================================================================
// DualLattice Coboundary
// ============================================================================

#[test]
fn test_dual_coboundary_vertex() {
    // Coboundary of a vertex gives edges incident to that vertex
    let primal = Lattice::<2>::square_torus(3);
    let dual = DualLattice::new(primal);

    let vertex = LatticeCell::<2>::vertex([1, 1]);
    let coboundary = dual.coboundary(&vertex);

    // In 2D, a vertex has 4 incident edges (or fewer at boundaries)
    // For torus, exactly 4
    assert!(!coboundary.is_empty());
}

#[test]
fn test_dual_coboundary_edge() {
    // Coboundary of an edge gives faces incident to that edge
    let primal = Lattice::<2>::square_torus(3);
    let dual = DualLattice::new(primal);

    let edge = LatticeCell::<2>::edge([0, 0], 0);
    let coboundary = dual.coboundary(&edge);

    // In 2D, an edge has 2 incident faces
    assert!(!coboundary.is_empty());
}
