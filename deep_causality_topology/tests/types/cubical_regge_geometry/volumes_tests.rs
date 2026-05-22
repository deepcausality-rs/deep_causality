/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `CubicalReggeGeometry::cell_volume` and `top_cell_volume` — Phase R1.

use deep_causality_topology::utils_tests::{
    open_cube_3, open_square_3, per_axis_geometry, per_edge_uniform_per_axis, periodic_cube_3,
    periodic_square_3, unit_geometry,
};
use deep_causality_topology::{CubicalReggeGeometry, LatticeCell, LatticeComplex};

// -- Task 2.5: unit-edge invariants ------------------------------------------------

#[test]
fn unit_edge_every_cell_has_volume_one_in_open_square() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    for grade in 0..=2 {
        for cell in lattice.iter_cells(grade) {
            assert_eq!(geom.cell_volume(&lattice, &cell), 1.0);
        }
    }
}

#[test]
fn unit_edge_every_cell_has_volume_one_in_periodic_cube() {
    let lattice = periodic_cube_3();
    let geom = unit_geometry::<3>();
    for grade in 0..=3 {
        for cell in lattice.iter_cells(grade) {
            assert_eq!(geom.cell_volume(&lattice, &cell), 1.0);
        }
    }
}

#[test]
fn unit_edge_top_cell_volume_is_one_4d() {
    let lattice: LatticeComplex<4, f64> = LatticeComplex::hypercubic_torus(2);
    let geom = unit_geometry::<4>();
    for cell in lattice.iter_cells(4) {
        assert_eq!(geom.top_cell_volume(&lattice, &cell), 1.0);
    }
}

#[test]
fn unit_edge_with_f32_precision() {
    let lattice: LatticeComplex<2, f32> = LatticeComplex::square_open(2);
    let geom: CubicalReggeGeometry<2, f32> = CubicalReggeGeometry::unit();
    for cell in lattice.iter_cells(2) {
        assert_eq!(geom.cell_volume(&lattice, &cell), 1.0_f32);
    }
}

#[test]
fn unit_edge_vertex_volume_is_empty_product_one() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let v = LatticeCell::vertex([1, 1]);
    assert_eq!(geom.cell_volume(&lattice, &v), 1.0);
}

// -- Task 2.6: per-axis closed forms -----------------------------------------------

#[test]
fn per_axis_top_cube_volume_equals_product_of_axis_lengths_2d() {
    let lattice = open_square_3();
    let geom = per_axis_geometry([2.0, 3.0]);
    let cell = LatticeCell::new([0, 0], 0b11);
    assert_eq!(geom.top_cell_volume(&lattice, &cell), 6.0);
}

#[test]
fn per_axis_top_cube_volume_equals_product_of_axis_lengths_3d() {
    let lattice = open_cube_3();
    let geom = per_axis_geometry([2.0, 3.0, 5.0]);
    let cell = LatticeCell::new([0, 0, 0], 0b111);
    assert_eq!(geom.top_cell_volume(&lattice, &cell), 30.0);
}

#[test]
fn per_axis_edge_volume_equals_its_axis_length() {
    let lattice = open_square_3();
    let geom = per_axis_geometry([2.0, 7.0]);
    let edge_along_axis_0 = LatticeCell::edge([0, 0], 0);
    let edge_along_axis_1 = LatticeCell::edge([0, 0], 1);
    assert_eq!(geom.cell_volume(&lattice, &edge_along_axis_0), 2.0);
    assert_eq!(geom.cell_volume(&lattice, &edge_along_axis_1), 7.0);
}

#[test]
fn per_axis_face_volume_in_3d_equals_product_of_active_axes() {
    let lattice = open_cube_3();
    let geom = per_axis_geometry([2.0, 3.0, 5.0]);
    // 2-face spanning axes 0 and 2 (orientation 0b101).
    let face = LatticeCell::new([0, 0, 0], 0b101);
    assert_eq!(geom.cell_volume(&lattice, &face), 10.0);
}

// -- Task 2.7: per-edge reproduces per-axis under uniform-per-axis assignment -----

#[test]
fn per_edge_uniform_matches_per_axis_2d() {
    let lattice = open_square_3();
    let axis_lengths = [2.0, 3.0];
    let per_axis = per_axis_geometry::<2>(axis_lengths);
    let per_edge = per_edge_uniform_per_axis(&lattice, axis_lengths);

    for grade in 0..=2 {
        for cell in lattice.iter_cells(grade) {
            let va = per_axis.cell_volume(&lattice, &cell);
            let ve = per_edge.cell_volume(&lattice, &cell);
            let tol = f64::EPSILON * 8.0 * va.abs().max(1.0);
            assert!(
                (va - ve).abs() <= tol,
                "grade {grade} cell {:?} ori {:#b}: per_axis={va} vs per_edge={ve}",
                cell.position(),
                cell.orientation(),
            );
        }
    }
}

#[test]
fn per_edge_uniform_matches_per_axis_3d() {
    let lattice = open_cube_3();
    let axis_lengths = [2.0, 3.0, 5.0];
    let per_axis = per_axis_geometry::<3>(axis_lengths);
    let per_edge = per_edge_uniform_per_axis(&lattice, axis_lengths);

    for grade in 0..=3 {
        for cell in lattice.iter_cells(grade) {
            let va = per_axis.cell_volume(&lattice, &cell);
            let ve = per_edge.cell_volume(&lattice, &cell);
            let tol = f64::EPSILON * 8.0 * va.abs().max(1.0);
            assert!(
                (va - ve).abs() <= tol,
                "grade {grade} cell {:?} ori {:#b}: per_axis={va} vs per_edge={ve}",
                cell.position(),
                cell.orientation(),
            );
        }
    }
}

#[test]
fn per_edge_uniform_matches_per_axis_periodic_2d() {
    let lattice = periodic_square_3();
    let axis_lengths = [2.5, 7.5];
    let per_axis = per_axis_geometry::<2>(axis_lengths);
    let per_edge = per_edge_uniform_per_axis(&lattice, axis_lengths);

    for cell in lattice.iter_cells(2) {
        let va = per_axis.top_cell_volume(&lattice, &cell);
        let ve = per_edge.top_cell_volume(&lattice, &cell);
        assert!((va - ve).abs() <= f64::EPSILON * 16.0);
    }
}

// -- Uniform path (single-spacing isotropic) ---------------------------------------

#[test]
fn uniform_top_cube_volume_is_length_to_power_d() {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_open(3);
    let geom: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::uniform(0.5);
    let cell = LatticeCell::new([0, 0, 0], 0b111);
    let expected = 0.5_f64.powi(3);
    let got = geom.top_cell_volume(&lattice, &cell);
    assert!((expected - got).abs() <= f64::EPSILON * 4.0);
}

#[test]
#[should_panic(expected = "top_cell_volume requires a D-cell")]
fn top_cell_volume_panics_on_wrong_grade() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let edge = LatticeCell::edge([0, 0], 0); // grade 1, not D=2
    let _ = geom.top_cell_volume(&lattice, &edge);
}
