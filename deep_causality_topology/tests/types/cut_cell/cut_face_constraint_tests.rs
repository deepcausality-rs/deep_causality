/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Aperture-resolved cut-face no-slip constraint geometry (`add-aperture-resolved-noslip`,
//! Group A). `CutCellRegistry::cut_face_constraints` derives, per `Cut` cell, the wall condition
//! at the wetted fragment as aperture-weighted constraint rows — the smooth replacement for the
//! staircase `solid_incident_edges`.
//!
//! These tests pin the *row data* (which edges, which weights, the normal/tangential split) on
//! known cut geometries, across `f64` and `Float106`. The constrained projector consuming the rows
//! and driving the fragment velocity to zero is the Group B single-cut-cell gate.

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_topology::{
    ChainComplex, CutCell, CutCellRegistry, CutConstraintKind, CutFaceFragment, LatticeCell,
    LatticeComplex, SourceGeometry,
};

fn tol<R: RealField + FromPrimitive>() -> R {
    R::from_f64(1e-10).unwrap()
}

// -- emptiness: full Fluid / Solid cells contribute no cut-face rows -----------------------------

#[test]
fn empty_registry_emits_no_cut_face_rows() {
    let lattice = LatticeComplex::<2, f64>::square_open(4);
    let reg = CutCellRegistry::<2, f64>::new();
    assert!(reg.cut_face_constraints(&lattice).is_empty());
}

#[test]
fn full_fluid_and_solid_cells_emit_no_cut_face_rows() {
    // A registry holding only explicit Fluid and Solid cells (no Cut cells) yields no rows: the
    // cut-face path is for partially-wetted cells; a Solid cell's interior pin is the staircase
    // path's job (spec scenario "Fully-fluid and fully-solid cells are unaffected").
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let top = |base: [usize; 2]| {
        lattice
            .cells(2)
            .position(|c| *c.position() == base)
            .unwrap()
    };
    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(top([1, 1]), CutCell::<2, f64>::solid(1.0));
    reg.insert(top([2, 2]), CutCell::<2, f64>::fluid(1.0));
    assert!(reg.cut_face_constraints(&lattice).is_empty());
}

// -- 2D mechanics: an axis-aligned cut emits one no-penetration + one tangential row -------------

fn edge_index<const D: usize, R: RealField>(
    lattice: &LatticeComplex<D, R>,
    axis: usize,
    pos: [usize; D],
) -> usize {
    lattice
        .cells(1)
        .position(|c| c.orientation().trailing_zeros() as usize == axis && *c.position() == pos)
        .unwrap()
}

/// A cut cell at `[1,1]` with a `+y` wall (solid below), `x`-faces half-wet, `y`-low face dry and
/// `y`-high face wet. The no-penetration row reconstructs `u_y` from the two `y`-edges (each ½);
/// the tangential row reconstructs `u_x` from the single wet `x`-edge (the dry-bounded one drops).
fn axis_aligned_cut_emits_normal_and_tangential_rows<R: RealField + FromPrimitive>() {
    let lattice = LatticeComplex::<2, R>::square_open(4);
    let cell = LatticeCell::<2>::new([1, 1], 0b11);
    let idx = lattice.cells(2).position(|c| c == cell).unwrap();

    let normal = [R::zero(), R::one()];
    let centroid = [R::zero(), R::zero()];
    let fragment = CutFaceFragment::<2, R>::new(R::one(), normal, centroid, SourceGeometry::Plane);
    let apertures = [
        [R::from_f64(0.5).unwrap(), R::from_f64(0.5).unwrap()],
        [R::zero(), R::one()],
    ];
    let mut reg = CutCellRegistry::<2, R>::new();
    reg.insert(
        idx,
        CutCell::<2, R>::cut(
            R::one(),
            R::from_f64(0.5).unwrap(),
            apertures,
            vec![fragment],
        ),
    );

    let rows = reg.cut_face_constraints(&lattice);
    assert_eq!(
        rows.len(),
        2,
        "2D wall: 1 no-penetration + 1 tangential row"
    );

    let no_pen = rows
        .iter()
        .find(|r| r.kind() == CutConstraintKind::NoPenetration)
        .expect("a no-penetration row");
    let tang = rows
        .iter()
        .find(|r| r.kind() == CutConstraintKind::Tangential)
        .expect("a tangential row");

    // Row measure is the fragment area; static-body target is zero.
    assert!((no_pen.row_weight() - R::one()).abs() < tol::<R>());
    assert!(no_pen.target().abs() < tol::<R>());

    // No-penetration (n̂ = +y): the two y-edges, each weight ½.
    let y_lo = edge_index(&lattice, 1, [1, 1]);
    let y_hi = edge_index(&lattice, 1, [2, 1]);
    let mut np = no_pen.entries().to_vec();
    np.sort_by_key(|(e, _)| *e);
    let mut expect_np = [(y_lo, 0.5_f64), (y_hi, 0.5_f64)];
    expect_np.sort_by_key(|(e, _)| *e);
    assert_eq!(np.len(), 2);
    for ((e, w), (ee, ew)) in np.iter().zip(expect_np.iter()) {
        assert_eq!(e, ee);
        assert!((*w - R::from_f64(*ew).unwrap()).abs() < tol::<R>());
    }

    // Tangential (t̂ = ±x): only the wet x-edge at [1,2] survives (the y-low face is dry, so the
    // x-edge at [1,1] drops). This is the key "not the whole edge ring" property.
    let x_dry = edge_index(&lattice, 0, [1, 1]);
    let x_wet = edge_index(&lattice, 0, [1, 2]);
    assert_eq!(tang.entries().len(), 1, "the dry-bounded x-edge drops out");
    let (te, tw) = tang.entries()[0];
    assert_eq!(te, x_wet);
    assert!((tw.abs() - R::one()).abs() < tol::<R>());
    assert!(
        !tang.entries().iter().any(|(e, _)| *e == x_dry),
        "a dry-bounded edge must not be constrained"
    );
}

#[test]
fn axis_aligned_cut_emits_normal_and_tangential_rows_f64() {
    axis_aligned_cut_emits_normal_and_tangential_rows::<f64>();
}

#[test]
fn axis_aligned_cut_emits_normal_and_tangential_rows_float106() {
    axis_aligned_cut_emits_normal_and_tangential_rows::<Float106>();
}

// -- aperture weighting: a fully-wetted cut reduces to the uniform cell-centre average -----------

#[test]
fn fully_wetted_cut_gives_uniform_cell_centre_weights() {
    // Every aperture 1 ⇒ each parallel edge weight is 1/2^{D-1} (the aperture-blind A1 average).
    let lattice = LatticeComplex::<2, f64>::square_open(4);
    let cell = LatticeCell::<2>::new([1, 1], 0b11);
    let idx = lattice.cells(2).position(|c| c == cell).unwrap();

    // A 45° normal so both axes carry a nonzero direction component.
    let s = 1.0 / 2.0_f64.sqrt();
    let fragment = CutFaceFragment::<2, f64>::new(1.0, [s, s], [0.0, 0.0], SourceGeometry::Plane);
    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(
        idx,
        CutCell::<2, f64>::cut(1.0, 0.5, [[1.0, 1.0], [1.0, 1.0]], vec![fragment]),
    );

    let rows = reg.cut_face_constraints(&lattice);
    let no_pen = rows
        .iter()
        .find(|r| r.kind() == CutConstraintKind::NoPenetration)
        .unwrap();
    // n̂ = (s, s): both axes appear, each with two edges of weight ½, scaled by the ±s direction.
    assert_eq!(no_pen.entries().len(), 4);
    for (_, w) in no_pen.entries() {
        assert!((w.abs() - s * 0.5).abs() < 1e-12, "weight |s·½|, got {w}");
    }
}

// -- 3D: a +z wall emits one no-penetration + two tangential rows -------------------------------

#[test]
fn three_d_axis_aligned_cut_emits_one_normal_two_tangential() {
    let lattice = LatticeComplex::<3, f64>::open([3, 3, 3]);
    let cell = LatticeCell::<3>::new([1, 1, 1], 0b111);
    let idx = lattice.cells(3).position(|c| c == cell).unwrap();

    let fragment = CutFaceFragment::<3, f64>::new(
        1.0,
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
        SourceGeometry::Plane,
    );
    let mut reg = CutCellRegistry::<3, f64>::new();
    reg.insert(
        idx,
        CutCell::<3, f64>::cut(
            1.0,
            0.5,
            [[1.0, 1.0], [1.0, 1.0], [0.0, 1.0]],
            vec![fragment],
        ),
    );

    let rows = reg.cut_face_constraints(&lattice);
    let n_pen = rows
        .iter()
        .filter(|r| r.kind() == CutConstraintKind::NoPenetration)
        .count();
    let n_tan = rows
        .iter()
        .filter(|r| r.kind() == CutConstraintKind::Tangential)
        .count();
    assert_eq!(n_pen, 1, "one no-penetration row");
    assert_eq!(n_tan, 2, "two tangential rows spanning the wall (D-1 = 2)");

    // The no-penetration row (n̂ = +z) reconstructs u_z from the cell's 4 z-edges.
    let no_pen = rows
        .iter()
        .find(|r| r.kind() == CutConstraintKind::NoPenetration)
        .unwrap();
    assert_eq!(no_pen.entries().len(), 4, "4 z-edges of the cube");
    for (_, w) in no_pen.entries() {
        assert!((w.abs() - 0.25).abs() < 1e-12, "uniform ¼ per z-edge");
    }
}
