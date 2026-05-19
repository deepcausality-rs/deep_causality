/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{HeavyHexLattice, KagomeLattice, LatticeComplex, TriangularLattice};

// ============================================================================
// Specialized Constructors - 2D LatticeComplex
// ============================================================================

#[test]
fn test_lattice_2d_square_torus() {
    let lattice = LatticeComplex::<2>::square_torus(5);

    assert_eq!(lattice.shape(), &[5, 5]);
    assert_eq!(lattice.periodic(), &[true, true]);
    assert_eq!(lattice.dim(), 2);
}

#[test]
fn test_lattice_2d_square_open() {
    let lattice = LatticeComplex::<2>::square_open(4);

    assert_eq!(lattice.shape(), &[4, 4]);
    assert_eq!(lattice.periodic(), &[false, false]);
    assert_eq!(lattice.dim(), 2);
}

// ============================================================================
// Specialized Constructors - 3D LatticeComplex
// ============================================================================

#[test]
fn test_lattice_3d_cubic_torus() {
    let lattice = LatticeComplex::<3>::cubic_torus(3);

    assert_eq!(lattice.shape(), &[3, 3, 3]);
    assert_eq!(lattice.periodic(), &[true, true, true]);
    assert_eq!(lattice.dim(), 3);
}

#[test]
fn test_lattice_3d_cubic_open() {
    let lattice = LatticeComplex::<3>::cubic_open(4);

    assert_eq!(lattice.shape(), &[4, 4, 4]);
    assert_eq!(lattice.periodic(), &[false, false, false]);
    assert_eq!(lattice.dim(), 3);
}

// ============================================================================
// Specialized Constructors - 4D LatticeComplex
// ============================================================================

#[test]
fn test_lattice_4d_hypercubic_torus() {
    let lattice = LatticeComplex::<4>::hypercubic_torus(2);

    assert_eq!(lattice.shape(), &[2, 2, 2, 2]);
    assert_eq!(lattice.periodic(), &[true, true, true, true]);
    assert_eq!(lattice.dim(), 4);
}

// ============================================================================
// Generic Constructors
// ============================================================================

#[test]
fn test_lattice_generic_torus() {
    let lattice = LatticeComplex::<3>::torus([2, 3, 4]);

    assert_eq!(lattice.shape(), &[2, 3, 4]);
    assert_eq!(lattice.periodic(), &[true, true, true]);
}

#[test]
fn test_lattice_generic_open() {
    let lattice = LatticeComplex::<2>::open([5, 10]);

    assert_eq!(lattice.shape(), &[5, 10]);
    assert_eq!(lattice.periodic(), &[false, false]);
}

#[test]
fn test_lattice_mixed_boundary() {
    // Create a lattice with mixed boundary conditions
    let lattice = LatticeComplex::<3>::new([4, 4, 4], [true, false, true]);

    assert_eq!(lattice.shape(), &[4, 4, 4]);
    assert_eq!(lattice.periodic(), &[true, false, true]);
}

// ============================================================================
// Heavy-Hex / Kagome / Triangular construction + getters
// ============================================================================

#[test]
fn test_heavy_hex_lattice_new_and_getters() {
    let lat = HeavyHexLattice::new(3, 5);
    assert_eq!(lat.rows(), 3);
    assert_eq!(lat.cols(), 5);
}

#[test]
fn test_heavy_hex_lattice_zero_dims() {
    let lat = HeavyHexLattice::new(0, 0);
    assert_eq!(lat.rows(), 0);
    assert_eq!(lat.cols(), 0);
}

#[test]
fn test_kagome_lattice_new_and_getters() {
    let lat = KagomeLattice::new([4, 6], [true, false]);
    assert_eq!(lat.size(), &[4, 6]);
    assert_eq!(lat.periodic(), &[true, false]);
}

#[test]
fn test_kagome_lattice_periodic() {
    let lat = KagomeLattice::new([2, 2], [true, true]);
    assert_eq!(lat.periodic(), &[true, true]);
    assert_eq!(lat.size()[0], 2);
    assert_eq!(lat.size()[1], 2);
}

#[test]
fn test_triangular_lattice_new_and_getters() {
    let lat = TriangularLattice::new([7, 8], [false, true]);
    assert_eq!(lat.size(), &[7, 8]);
    assert_eq!(lat.periodic(), &[false, true]);
}

#[test]
fn test_triangular_lattice_open() {
    let lat = TriangularLattice::new([3, 3], [false, false]);
    assert_eq!(lat.periodic(), &[false, false]);
    assert_eq!(lat.size(), &[3, 3]);
}
