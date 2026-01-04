/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{Cell, LatticeCell};

// ============================================================================
// LatticeCell Constructors
// ============================================================================

#[test]
fn test_lattice_cell_new() {
    let cell = LatticeCell::<3>::new([1, 2, 3], 0b101);

    assert_eq!(cell.position(), &[1, 2, 3]);
    assert_eq!(cell.orientation(), 0b101);
}

#[test]
fn test_lattice_cell_vertex() {
    let vertex = LatticeCell::<2>::vertex([5, 10]);

    assert_eq!(vertex.position(), &[5, 10]);
    assert_eq!(vertex.orientation(), 0);
    assert!(vertex.is_vertex());
}

#[test]
fn test_lattice_cell_edge() {
    // Edge along dimension 0 (x-axis)
    let edge_x = LatticeCell::<3>::edge([0, 0, 0], 0);
    assert_eq!(edge_x.orientation(), 0b001);
    assert!(edge_x.is_edge());

    // Edge along dimension 1 (y-axis)
    let edge_y = LatticeCell::<3>::edge([0, 0, 0], 1);
    assert_eq!(edge_y.orientation(), 0b010);
    assert!(edge_y.is_edge());

    // Edge along dimension 2 (z-axis)
    let edge_z = LatticeCell::<3>::edge([0, 0, 0], 2);
    assert_eq!(edge_z.orientation(), 0b100);
    assert!(edge_z.is_edge());
}

// ============================================================================
// LatticeCell Getters
// ============================================================================

#[test]
fn test_lattice_cell_position() {
    let cell = LatticeCell::<4>::new([1, 2, 3, 4], 0b1111);
    assert_eq!(cell.position(), &[1, 2, 3, 4]);
}

#[test]
fn test_lattice_cell_cell_dim() {
    // 0-cell (vertex) has dimension 0
    let vertex = LatticeCell::<3>::vertex([0, 0, 0]);
    assert_eq!(vertex.cell_dim(), 0);

    // 1-cell (edge) has dimension 1
    let edge = LatticeCell::<3>::edge([0, 0, 0], 1);
    assert_eq!(edge.cell_dim(), 1);

    // 2-cell (face) has dimension 2
    let face = LatticeCell::<3>::new([0, 0, 0], 0b011);
    assert_eq!(face.cell_dim(), 2);

    // 3-cell (volume) has dimension 3
    let volume = LatticeCell::<3>::new([0, 0, 0], 0b111);
    assert_eq!(volume.cell_dim(), 3);
}

// ============================================================================
// LatticeCell Predicates
// ============================================================================

#[test]
fn test_lattice_cell_is_vertex() {
    let vertex = LatticeCell::<2>::new([0, 0], 0);
    assert!(vertex.is_vertex());

    let edge = LatticeCell::<2>::new([0, 0], 1);
    assert!(!edge.is_vertex());
}

#[test]
fn test_lattice_cell_is_edge() {
    let edge1 = LatticeCell::<3>::new([0, 0, 0], 0b001);
    assert!(edge1.is_edge());

    let edge2 = LatticeCell::<3>::new([0, 0, 0], 0b010);
    assert!(edge2.is_edge());

    let vertex = LatticeCell::<3>::new([0, 0, 0], 0);
    assert!(!vertex.is_edge());

    let face = LatticeCell::<3>::new([0, 0, 0], 0b011);
    assert!(!face.is_edge());
}

#[test]
fn test_lattice_cell_is_face() {
    let face_xy = LatticeCell::<3>::new([0, 0, 0], 0b011);
    assert!(face_xy.is_face());

    let face_xz = LatticeCell::<3>::new([0, 0, 0], 0b101);
    assert!(face_xz.is_face());

    let face_yz = LatticeCell::<3>::new([0, 0, 0], 0b110);
    assert!(face_yz.is_face());

    let volume = LatticeCell::<3>::new([0, 0, 0], 0b111);
    assert!(!volume.is_face());
}

// ============================================================================
// LatticeCell Operations
// ============================================================================

#[test]
fn test_lattice_cell_vertices_vertex() {
    // A vertex has only itself as vertex
    let vertex = LatticeCell::<2>::vertex([3, 4]);
    let vertices = vertex.vertices();

    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0], [3, 4]);
}

#[test]
fn test_lattice_cell_vertices_edge() {
    // An edge has 2 vertices
    let edge = LatticeCell::<2>::edge([1, 2], 0); // x-direction
    let vertices = edge.vertices();

    assert_eq!(vertices.len(), 2);
    assert!(vertices.contains(&[1, 2]));
    assert!(vertices.contains(&[2, 2]));
}

#[test]
fn test_lattice_cell_vertices_face() {
    // A face (2-cell) has 4 vertices
    let face = LatticeCell::<2>::new([0, 0], 0b11); // xy-plane
    let vertices = face.vertices();

    assert_eq!(vertices.len(), 4);
    assert!(vertices.contains(&[0, 0]));
    assert!(vertices.contains(&[1, 0]));
    assert!(vertices.contains(&[0, 1]));
    assert!(vertices.contains(&[1, 1]));
}

#[test]
fn test_lattice_cell_vertices_cube() {
    // A 3-cube has 8 vertices
    let cube = LatticeCell::<3>::new([0, 0, 0], 0b111);
    let vertices = cube.vertices();

    assert_eq!(vertices.len(), 8);
}

// ============================================================================
// LatticeCell Cell trait
// ============================================================================

#[test]
fn test_lattice_cell_dim_trait() {
    let edge = LatticeCell::<3>::edge([0, 0, 0], 1);
    assert_eq!(Cell::dim(&edge), 1);

    let face = LatticeCell::<3>::new([0, 0, 0], 0b101);
    assert_eq!(Cell::dim(&face), 2);
}

#[test]
fn test_lattice_cell_boundary_vertex() {
    // Vertices have empty boundary
    let vertex = LatticeCell::<2>::vertex([0, 0]);
    let boundary = Cell::boundary(&vertex);

    assert!(boundary.is_empty());
}

#[test]
fn test_lattice_cell_boundary_edge() {
    // An edge has 2 vertices as boundary
    let edge = LatticeCell::<2>::edge([0, 0], 0); // x-direction
    let boundary = Cell::boundary(&edge);

    assert_eq!(boundary.len(), 2);
    // Front and back vertices with opposite signs
    let coeffs: Vec<i8> = boundary.iter().map(|(_, c)| *c).collect();
    assert!(coeffs.contains(&1));
    assert!(coeffs.contains(&-1));
}

#[test]
fn test_lattice_cell_boundary_face() {
    // A face has 4 edges as boundary
    let face = LatticeCell::<2>::new([0, 0], 0b11); // xy-plane
    let boundary = Cell::boundary(&face);

    // 2D face: 2 dimensions * 2 faces = 4 edges
    assert_eq!(boundary.len(), 4);
}
