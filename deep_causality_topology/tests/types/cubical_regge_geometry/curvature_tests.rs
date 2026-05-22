/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `CubicalReggeGeometry::dihedral_angle`, `deficit_angle`, and `regge_action`
//! — Phases R2 (tasks 3.8–3.10) and R3 (tasks 4.6–4.11).

use deep_causality_topology::utils_tests::{
    open_cube_3, open_square_3, per_axis_geometry, per_edge_uniform_per_axis, periodic_cube_3,
    periodic_square_3, unit_geometry,
};
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeCell, LatticeComplex};

use std::f64::consts::{FRAC_PI_2, PI};

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

// ===== Phase R3 — Deficit angles + Regge action ==================================

// -- Task 4.6: open lattice — interior deficit 0, boundary deficit by formula -----

#[test]
fn open_2d_interior_vertex_deficit_is_zero() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let (hid, _) = find_cell(&lattice, 0, &LatticeCell::vertex([1, 1]));
    assert_eq!(geom.deficit_angle(&lattice, hid), 0.0);
}

#[test]
fn open_2d_corner_vertex_deficit_is_3pi_over_2() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let (hid, _) = find_cell(&lattice, 0, &LatticeCell::vertex([0, 0]));
    let want = 3.0 * FRAC_PI_2;
    let got = geom.deficit_angle(&lattice, hid);
    assert!((got - want).abs() <= f64::EPSILON * 4.0);
}

#[test]
fn open_2d_edge_vertex_deficit_is_pi() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let (hid, _) = find_cell(&lattice, 0, &LatticeCell::vertex([1, 0]));
    let got = geom.deficit_angle(&lattice, hid);
    assert!((got - PI).abs() <= f64::EPSILON * 4.0);
}

#[test]
fn open_2d_deficit_matches_4_minus_n_times_pi_over_2_everywhere() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    for hid in 0..lattice.num_cells(0) {
        let n = lattice.hinge_top_cube_neighbors(hid).len();
        let want = (4 - n as i32) as f64 * FRAC_PI_2;
        let got = geom.deficit_angle(&lattice, hid);
        assert!(
            (got - want).abs() <= f64::EPSILON * 4.0,
            "hinge {hid} n={n}: got {got}, want {want}",
        );
    }
}

// -- Task 4.7: periodic unit-edge → every deficit 0, action 0 ----------------------

#[test]
fn periodic_2d_unit_edge_all_deficits_are_zero() {
    let lattice = periodic_square_3();
    let geom = unit_geometry::<2>();
    for hid in 0..lattice.num_cells(0) {
        assert_eq!(geom.deficit_angle(&lattice, hid), 0.0);
    }
}

#[test]
fn periodic_2d_unit_edge_action_is_zero() {
    let lattice = periodic_square_3();
    let geom = unit_geometry::<2>();
    assert_eq!(geom.regge_action(&lattice), 0.0);
}

#[test]
fn periodic_3d_unit_edge_action_is_zero() {
    let lattice = periodic_cube_3();
    let geom = unit_geometry::<3>();
    assert_eq!(geom.regge_action(&lattice), 0.0);
}

#[test]
fn periodic_4d_unit_edge_action_is_zero() {
    let lattice: LatticeComplex<4, f64> = LatticeComplex::hypercubic_torus(2);
    let geom: CubicalReggeGeometry<4, f64> = CubicalReggeGeometry::unit();
    assert_eq!(geom.regge_action(&lattice), 0.0);
}

// -- Task 4.8: edge-length perturbation does not change any deficit ----------------

#[test]
fn deficit_is_invariant_under_per_axis_perturbation_2d() {
    let lattice = open_square_3();
    let unit = unit_geometry::<2>();
    let stretched = per_axis_geometry::<2>([2.0, 5.0]);
    for hid in 0..lattice.num_cells(0) {
        let a = unit.deficit_angle(&lattice, hid);
        let b = stretched.deficit_angle(&lattice, hid);
        assert!((a - b).abs() <= f64::EPSILON * 4.0);
    }
}

#[test]
fn deficit_is_invariant_under_per_edge_perturbation_3d() {
    let lattice = open_cube_3();
    let unit = unit_geometry::<3>();
    let per_edge = per_edge_uniform_per_axis(&lattice, [2.0, 3.0, 5.0]);
    for hid in 0..lattice.num_cells(1) {
        let a = unit.deficit_angle(&lattice, hid);
        let b = per_edge.deficit_angle(&lattice, hid);
        assert!((a - b).abs() <= f64::EPSILON * 4.0);
    }
}

// -- Task 4.9: open vs periodic action difference = boundary contribution ----------

#[test]
fn open_2d_unit_edge_action_equals_closed_form_10pi() {
    // 3×3 open lattice: 4 corners (deficit 3π/2), 4 edge-vertices (deficit π),
    // 1 interior (deficit 0). Vertex volume = 1 in 2D.
    // Action = 4·1·(3π/2) + 4·1·π + 1·1·0 = 6π + 4π = 10π.
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let got = geom.regge_action(&lattice);
    let want = 10.0 * PI;
    assert!(
        (got - want).abs() <= f64::EPSILON * 32.0,
        "got {got}, want {want}",
    );
}

#[test]
fn open_2d_minus_periodic_2d_equals_boundary_contribution() {
    let open = open_square_3();
    let periodic = periodic_square_3();
    let geom = unit_geometry::<2>();
    let diff = geom.regge_action(&open) - geom.regge_action(&periodic);
    let want = 10.0 * PI;
    assert!((diff - want).abs() <= f64::EPSILON * 32.0);
}

// -- R5.6: Lorentzian variant has its own action, real part agrees with Euclidean --

#[test]
fn lorentzian_regge_action_real_part_equals_euclidean_action() {
    // Design.md Decision 3 reduction check: on identical edge-length data, the
    // Lorentzian regge action's real part equals the Euclidean action's value.
    // Wick rotation lives in the imaginary part: `S_R^Lorentzian = i · S_R^Euclidean`.
    let lattice = open_square_3();
    let euclidean: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let lorentzian = euclidean.clone().with_timelike_axes([true, false]).unwrap();
    let s_e = euclidean.regge_action(&lattice);
    let s_l = lorentzian.regge_action_lorentzian(&lattice);
    assert_eq!(
        s_l.re, 0.0,
        "Lorentzian action real part must be zero (purely imaginary)"
    );
    assert_eq!(
        s_l.im, s_e,
        "Lorentzian action imaginary part equals Euclidean action"
    );
}

#[test]
fn timelike_axes_does_not_affect_deficit_angle() {
    // Deficit angles are geometric (depend only on hinge incidence count) and
    // are signature-independent. Both Euclidean and Lorentzian variants return
    // identical deficit values.
    let lattice = open_square_3();
    let euclidean: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let lorentzian = euclidean.clone().with_timelike_axes([true, false]).unwrap();
    for hid in 0..lattice.num_cells(0) {
        assert_eq!(
            euclidean.deficit_angle(&lattice, hid),
            lorentzian.deficit_angle(&lattice, hid),
        );
    }
}

// -- Task 4.11: 3D PerAxis — volume factor flows through ----------------------------

#[test]
fn open_3d_per_axis_action_scales_with_edge_volumes() {
    // 3D hinges are edges (1-cells); their volume IS the per-axis edge length.
    // Differing per-axis lengths give a different (and larger) total action than
    // the unit case.
    let lattice = open_cube_3();
    let unit = unit_geometry::<3>();
    let scaled = per_axis_geometry::<3>([2.0, 3.0, 5.0]);
    let action_unit = unit.regge_action(&lattice);
    let action_scaled = scaled.regge_action(&lattice);
    assert!(action_unit > 0.0);
    assert!(action_scaled > action_unit);
}

#[test]
fn open_3d_per_axis_action_equals_per_edge_action() {
    let lattice = open_cube_3();
    let axis_lengths = [2.0, 3.0, 5.0];
    let per_axis = per_axis_geometry::<3>(axis_lengths);
    let per_edge = per_edge_uniform_per_axis(&lattice, axis_lengths);
    let a = per_axis.regge_action(&lattice);
    let e = per_edge.regge_action(&lattice);
    let tol = f64::EPSILON * 64.0 * a.abs().max(1.0);
    assert!((a - e).abs() <= tol, "per_axis={a} vs per_edge={e}");
}

// -- D < 2 degenerate cases (covers early-return paths) ---------------------------

#[test]
fn deficit_angle_returns_zero_for_d1_lattice() {
    let lattice: LatticeComplex<1, f64> = LatticeComplex::new([3], [false]);
    let geom: CubicalReggeGeometry<1, f64> = CubicalReggeGeometry::unit();
    // D=1 has no (D-2) = (-1) hinges; the function returns R::zero() unconditionally.
    assert_eq!(geom.deficit_angle(&lattice, 0), 0.0);
    assert_eq!(geom.deficit_angle(&lattice, 999), 0.0);
}

#[test]
fn regge_action_returns_zero_for_d1_lattice() {
    let lattice: LatticeComplex<1, f64> = LatticeComplex::new([3], [true]);
    let geom: CubicalReggeGeometry<1, f64> = CubicalReggeGeometry::unit();
    assert_eq!(geom.regge_action(&lattice), 0.0);
}

#[test]
fn deficit_angle_out_of_range_hinge_returns_zero() {
    let lattice = open_square_3();
    let geom = unit_geometry::<2>();
    let n = lattice.num_cells(0);
    assert_eq!(geom.deficit_angle(&lattice, n), 0.0);
    assert_eq!(geom.deficit_angle(&lattice, n + 100), 0.0);
}

// -- Debug-assertion panic paths (covers format-argument lines) -------------------

#[test]
#[should_panic(expected = "top_cube must be a D-cell")]
fn dihedral_angle_panics_when_top_cube_wrong_grade() {
    let lattice = periodic_square_3();
    let geom = unit_geometry::<2>();
    let not_a_top_cube = LatticeCell::vertex([0, 0]); // grade 0, not D=2
    let hinge = LatticeCell::vertex([1, 1]);
    let _ = geom.dihedral_angle(&lattice, &not_a_top_cube, &hinge);
}

#[test]
#[should_panic(expected = "hinge must be a (D-2)-cell")]
fn dihedral_angle_panics_when_hinge_wrong_grade() {
    let lattice = periodic_square_3();
    let geom = unit_geometry::<2>();
    let top = LatticeCell::new([0, 0], 0b11); // grade 2 ✓
    let not_a_hinge = LatticeCell::edge([0, 0], 0); // grade 1, want grade 0
    let _ = geom.dihedral_angle(&lattice, &top, &not_a_hinge);
}
