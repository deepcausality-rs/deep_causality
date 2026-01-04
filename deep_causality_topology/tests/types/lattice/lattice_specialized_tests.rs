/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::Lattice;

// ============================================================================
// Specialized Constructors - 2D Lattice
// ============================================================================

#[test]
fn test_lattice_2d_square_torus() {
    let lattice = Lattice::<2>::square_torus(5);

    assert_eq!(lattice.shape(), &[5, 5]);
    assert_eq!(lattice.periodic(), &[true, true]);
    assert_eq!(lattice.dim(), 2);
}

#[test]
fn test_lattice_2d_square_open() {
    let lattice = Lattice::<2>::square_open(4);

    assert_eq!(lattice.shape(), &[4, 4]);
    assert_eq!(lattice.periodic(), &[false, false]);
    assert_eq!(lattice.dim(), 2);
}

// ============================================================================
// Specialized Constructors - 3D Lattice
// ============================================================================

#[test]
fn test_lattice_3d_cubic_torus() {
    let lattice = Lattice::<3>::cubic_torus(3);

    assert_eq!(lattice.shape(), &[3, 3, 3]);
    assert_eq!(lattice.periodic(), &[true, true, true]);
    assert_eq!(lattice.dim(), 3);
}

#[test]
fn test_lattice_3d_cubic_open() {
    let lattice = Lattice::<3>::cubic_open(4);

    assert_eq!(lattice.shape(), &[4, 4, 4]);
    assert_eq!(lattice.periodic(), &[false, false, false]);
    assert_eq!(lattice.dim(), 3);
}

// ============================================================================
// Specialized Constructors - 4D Lattice
// ============================================================================

#[test]
fn test_lattice_4d_hypercubic_torus() {
    let lattice = Lattice::<4>::hypercubic_torus(2);

    assert_eq!(lattice.shape(), &[2, 2, 2, 2]);
    assert_eq!(lattice.periodic(), &[true, true, true, true]);
    assert_eq!(lattice.dim(), 4);
}

// ============================================================================
// Generic Constructors
// ============================================================================

#[test]
fn test_lattice_generic_torus() {
    let lattice = Lattice::<3>::torus([2, 3, 4]);

    assert_eq!(lattice.shape(), &[2, 3, 4]);
    assert_eq!(lattice.periodic(), &[true, true, true]);
}

#[test]
fn test_lattice_generic_open() {
    let lattice = Lattice::<2>::open([5, 10]);

    assert_eq!(lattice.shape(), &[5, 10]);
    assert_eq!(lattice.periodic(), &[false, false]);
}

#[test]
fn test_lattice_mixed_boundary() {
    // Create a lattice with mixed boundary conditions
    let lattice = Lattice::<3>::new([4, 4, 4], [true, false, true]);

    assert_eq!(lattice.shape(), &[4, 4, 4]);
    assert_eq!(lattice.periodic(), &[true, false, true]);
}
