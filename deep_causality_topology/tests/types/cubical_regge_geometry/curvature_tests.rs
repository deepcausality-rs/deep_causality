/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `CubicalReggeGeometry::dihedral_angle` — Phase R2 tasks 3.8–3.10.
//!
//! Deficit-angle and Regge-action tests land in Phase R3.

use deep_causality_topology::utils_tests::{
    open_square_3, per_axis_geometry, per_edge_uniform_per_axis, periodic_cube_3,
    periodic_square_3, unit_geometry,
};
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeCell, LatticeComplex};

use std::f64::consts::FRAC_PI_2;

/// Locate the cell at (position, orientation) in the canonical iter order.
fn find_cell<const D: usize, R>(
    lattice: &LatticeComplex<D, R>,
    grade: usize,
    target: &LatticeCell<D>,
) -> (usize, LatticeCell<D>)
where
    R: deep_causality_num::RealField,
{
    let id = lattice
        .cells(grade)
        .position(|c| c == *target)
        .expect("cell not found");
    (id, target.clone())
}

fn fetch_cell<const D: usize, R>(
    lattice: &LatticeComplex<D, R>,
    grade: usize,
    id: usize,
) -> LatticeCell<D>
where
    R: deep_causality_num::RealField,
{
    lattice.cells(grade).nth(id).expect("cell id out of range")
}

// -- Task 3.8: unit-edge dihedrals are π/2 -----------------------------------------

#[test]
fn unit_edge_every_dihedral_is_pi_over_two_in_periodic_2d() {
    let lattice = periodic_square_3();
    let geom = unit_geometry::<2>();
    for hinge_id in 0..lattice.num_cells(0) {
        let hinge = fetch_cell(&lattice, 0, hinge_id);
        for top_id in lattice.hinge_top_cube_neighbors(hinge_id) {
            let top = fetch_cell(&lattice, 2, top_id);
            let theta = geom.dihedral_angle(&lattice, &top, &hinge);
            assert!((theta - FRAC_PI_2).abs() <= f64::EPSILON * 4.0);
        }
    }
}

#[test]
fn unit_edge_every_dihedral_is_pi_over_two_in_periodic_3d() {
    let lattice = periodic_cube_3();
    let geom = unit_geometry::<3>();
    for hinge_id in 0..lattice.num_cells(1) {
        let hinge = fetch_cell(&lattice, 1, hinge_id);
        for top_id in lattice.hinge_top_cube_neighbors(hinge_id) {
            let top = fetch_cell(&lattice, 3, top_id);
            let theta = geom.dihedral_angle(&lattice, &top, &hinge);
            assert!((theta - FRAC_PI_2).abs() <= f64::EPSILON * 4.0);
        }
    }
}

#[test]
fn unit_edge_dihedral_is_pi_over_two_in_4d() {
    let lattice: LatticeComplex<4, f64> = LatticeComplex::hypercubic_torus(2);
    let geom: CubicalReggeGeometry<4, f64> = CubicalReggeGeometry::unit();
    let hinge = fetch_cell(&lattice, 2, 0);
    let top_ids = lattice.hinge_top_cube_neighbors(0);
    assert!(!top_ids.is_empty(), "expected incident 4-cubes");
    for top_id in top_ids {
        let top = fetch_cell(&lattice, 4, top_id);
        let theta = geom.dihedral_angle(&lattice, &top, &hinge);
        assert!((theta - FRAC_PI_2).abs() <= f64::EPSILON * 4.0);
    }
}

#[test]
fn unit_edge_dihedral_in_f32_precision() {
    let lattice: LatticeComplex<2, f32> = LatticeComplex::square_torus(3);
    let geom: CubicalReggeGeometry<2, f32> = CubicalReggeGeometry::unit();
    let hinge = fetch_cell(&lattice, 0, 0);
    let top_id = lattice.hinge_top_cube_neighbors(0)[0];
    let top = fetch_cell(&lattice, 2, top_id);
    let theta = geom.dihedral_angle(&lattice, &top, &hinge);
    assert!((theta - std::f32::consts::FRAC_PI_2).abs() <= f32::EPSILON * 4.0);
}

// -- Task 3.9: interior-vertex dihedral sum is 2π ----------------------------------

#[test]
fn periodic_2d_interior_vertex_dihedral_sum_is_2pi_per_axis() {
    let lattice = periodic_square_3();
    let geom = per_axis_geometry::<2>([2.0, 3.0]);

    let (hinge_id, hinge) = find_cell(&lattice, 0, &LatticeCell::vertex([1, 1]));
    let sum: f64 = lattice
        .hinge_top_cube_neighbors(hinge_id)
        .into_iter()
        .map(|tid| {
            let top = fetch_cell(&lattice, 2, tid);
            geom.dihedral_angle(&lattice, &top, &hinge)
        })
        .sum();
    let two_pi = std::f64::consts::TAU;
    assert!(
        (sum - two_pi).abs() <= f64::EPSILON * 8.0,
        "per-axis sum at interior vertex: got {sum}, want {two_pi}",
    );
}

#[test]
fn periodic_3d_interior_edge_dihedral_sum_is_2pi_per_axis() {
    let lattice = periodic_cube_3();
    let geom = per_axis_geometry::<3>([1.5, 2.5, 3.5]);

    let (hinge_id, hinge) = find_cell(&lattice, 1, &LatticeCell::edge([1, 1, 1], 0));
    let sum: f64 = lattice
        .hinge_top_cube_neighbors(hinge_id)
        .into_iter()
        .map(|tid| {
            let top = fetch_cell(&lattice, 3, tid);
            geom.dihedral_angle(&lattice, &top, &hinge)
        })
        .sum();
    let two_pi = std::f64::consts::TAU;
    assert!((sum - two_pi).abs() <= f64::EPSILON * 8.0);
}

#[test]
fn open_2d_boundary_vertex_dihedral_sum_is_less_than_2pi() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();

    // Corner vertex has one incident square: sum = π/2.
    let (hinge_id, hinge) = find_cell(&lattice, 0, &LatticeCell::vertex([0, 0]));
    let sum: f64 = lattice
        .hinge_top_cube_neighbors(hinge_id)
        .into_iter()
        .map(|tid| {
            let top = fetch_cell(&lattice, 2, tid);
            geom.dihedral_angle(&lattice, &top, &hinge)
        })
        .sum();
    assert!((sum - FRAC_PI_2).abs() <= f64::EPSILON * 4.0);
}

// -- Task 3.10: per-edge / per-axis dihedral agreement -----------------------------

#[test]
fn per_edge_uniform_matches_per_axis_dihedral_2d() {
    let lattice = periodic_square_3();
    let axis_lengths = [2.0, 3.0];
    let per_axis = per_axis_geometry::<2>(axis_lengths);
    let per_edge = per_edge_uniform_per_axis(&lattice, axis_lengths);

    for hinge_id in 0..lattice.num_cells(0) {
        let hinge = fetch_cell(&lattice, 0, hinge_id);
        for top_id in lattice.hinge_top_cube_neighbors(hinge_id) {
            let top = fetch_cell(&lattice, 2, top_id);
            let a = per_axis.dihedral_angle(&lattice, &top, &hinge);
            let e = per_edge.dihedral_angle(&lattice, &top, &hinge);
            assert!((a - e).abs() <= f64::EPSILON * 4.0);
        }
    }
}

#[test]
fn per_edge_uniform_matches_per_axis_dihedral_3d() {
    let lattice = periodic_cube_3();
    let axis_lengths = [1.5, 2.5, 3.5];
    let per_axis = per_axis_geometry::<3>(axis_lengths);
    let per_edge = per_edge_uniform_per_axis(&lattice, axis_lengths);

    // Spot-check a handful of hinges (full sweep is exhaustive but slow on 3x3x3).
    let max_check = 8.min(lattice.num_cells(1));
    for hinge_id in 0..max_check {
        let hinge = fetch_cell(&lattice, 1, hinge_id);
        for top_id in lattice.hinge_top_cube_neighbors(hinge_id) {
            let top = fetch_cell(&lattice, 3, top_id);
            let a = per_axis.dihedral_angle(&lattice, &top, &hinge);
            let e = per_edge.dihedral_angle(&lattice, &top, &hinge);
            assert!((a - e).abs() <= f64::EPSILON * 4.0);
        }
    }
}
