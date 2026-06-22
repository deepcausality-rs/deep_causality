/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for under-exercised cut-cell arms: the degenerate
//! `volume_fraction`, the `Primitive` constructors / `source` tags, the
//! negative-normal half-space reduction, the cylinder/disk fluid & solid early
//! exits, `CutCellRegistry::from_map`, the `clipped_cell_volume` cell-merging
//! floor, the cut-face-constraint zero-area / empty-fragment skips, and the
//! aperture-weighted reconstruction uniform fallback.

use deep_causality_topology::{
    CellClass, ChainComplex, CubicalReggeGeometry, CutCell, CutCellRegistry, CutConstraintKind,
    CutFaceFragment, LatticeCell, LatticeComplex, Primitive, SourceGeometry,
};
use std::collections::HashMap;

const TOL: f64 = 1e-10;

// -- carrier.rs: degenerate zero-volume fraction -------------------------------------

#[test]
fn volume_fraction_zero_full_volume_is_zero_not_nan() {
    // A degenerate cell with zero full volume must report fraction 0, not divide
    // by zero (the guarded branch).
    let c = CutCell::<2, f64>::cut(0.0, 0.0, [[0.0, 0.0], [0.0, 0.0]], Vec::new());
    assert_eq!(c.volume_fraction(), 0.0);
    assert!(c.volume_fraction().is_finite());
}

// -- primitive.rs: constructors and source tags --------------------------------------

#[test]
fn halfspace_zero_normal_is_returned_unchanged() {
    // A zero normal cannot be normalised, so `halfspace` returns it as-is (the
    // `norm2 <= 0` guard).
    let p = Primitive::<2, f64>::halfspace([0.0, 0.0], 0.7);
    match p {
        Primitive::Halfspace { normal, offset } => {
            assert_eq!(normal, [0.0, 0.0]);
            assert_eq!(offset, 0.7);
        }
        _ => panic!("expected Halfspace"),
    }
}

#[test]
fn primitive_source_tags() {
    // Each primitive's `source()` must return its geometry tag.
    let hs = Primitive::<2, f64>::halfspace([1.0, 0.0], 0.5);
    assert_eq!(hs.source(), SourceGeometry::Plane);

    let cyl = Primitive::<3, f64>::cylinder(2, [1.0, 1.0, 0.0], 0.5);
    assert_eq!(cyl.source(), SourceGeometry::Cylinder);

    let ball = Primitive::<2, f64>::ball([0.5, 0.5], 0.25);
    assert_eq!(ball.source(), SourceGeometry::Sphere);
}

// -- geometry.rs / intersection.rs: negative-normal half-space ------------------------

#[test]
fn halfspace_negative_normal_component_clips_correctly() {
    // A half-space normal with a negative component drives the `ni < 0`
    // reflection arm of `reduce_halfspace`. Solid { -x ≤ -0.3 } ⇔ { x ≥ 0.3 },
    // so in the unit cell the solid measure is 0.7 and the fluid is 0.3.
    let prim = Primitive::<2, f64>::halfspace([-1.0, 0.0], -0.3);
    let cell = CutCell::from_box(&prim, [0.0, 0.0], [1.0, 1.0]).unwrap();
    assert_eq!(cell.class(), CellClass::Cut);
    // Fluid is { -x ≥ -0.3 } ⇔ { x ≤ 0.3 } ⇒ measure 0.3.
    assert!((cell.fluid_volume() - 0.3).abs() < 1e-9);
}

// -- intersection.rs: cylinder fluid & solid early exits ------------------------------

#[test]
fn cylinder_far_from_cell_is_all_fluid() {
    // A cylinder whose disk does not reach the cell leaves the cell fully fluid
    // (the `solid <= eps` arm of `from_cylinder`).
    let prim = Primitive::<3, f64>::cylinder(2, [100.0, 100.0, 0.0], 0.5);
    let cell = CutCell::from_box(&prim, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).unwrap();
    assert_eq!(cell.class(), CellClass::Fluid);
}

#[test]
fn cylinder_engulfing_cell_is_all_solid() {
    // A large cylinder fully containing the cell makes the cell all solid (the
    // `fluid <= eps` arm of `from_cylinder`).
    let prim = Primitive::<3, f64>::cylinder(2, [0.5, 0.5, 0.0], 50.0);
    let cell = CutCell::from_box(&prim, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).unwrap();
    assert_eq!(cell.class(), CellClass::Solid);
}

#[test]
fn cylinder_through_cell_centre_is_cut() {
    // A cylinder centred at the cell centre cuts it (a fragment with a radial
    // outward normal is recorded — the `rn > 0` arm of `from_cylinder`).
    let prim = Primitive::<3, f64>::cylinder(2, [0.5, 0.5, 0.0], 0.3);
    let cell = CutCell::from_box(&prim, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).unwrap();
    assert_eq!(cell.class(), CellClass::Cut);
    assert!(cell.fluid_volume() > 0.0 && cell.fluid_volume() < 1.0);
    assert!(!cell.fragments().is_empty());
}

// -- intersection.rs: disk fragment (sphere source) -----------------------------------

#[test]
fn disk_through_cell_records_arc_fragment() {
    // A 2D disk cutting the cell records an arc fragment with a Sphere source
    // and a radial outward normal (the `arc > eps` / `rn > 0` arms of
    // `from_disk`).
    let prim = Primitive::<2, f64>::ball([0.5, 0.5], 0.3);
    let cell = CutCell::from_box(&prim, [0.0, 0.0], [1.0, 1.0]).unwrap();
    assert_eq!(cell.class(), CellClass::Cut);
    assert_eq!(cell.fragments().len(), 1);
    assert_eq!(cell.fragments()[0].source(), SourceGeometry::Sphere);
}

// -- registry.rs: from_map round-trip -------------------------------------------------

#[test]
fn registry_from_map_round_trips() {
    let mut map: HashMap<usize, CutCell<2, f64>> = HashMap::new();
    map.insert(5, CutCell::<2, f64>::solid(1.0));
    map.insert(9, CutCell::<2, f64>::fluid(1.0));
    let reg = CutCellRegistry::<2, f64>::from_map(map);

    assert_eq!(reg.len(), 2);
    assert!(reg.cell_merging_floor().is_none());
    assert_eq!(reg.get(5).map(|c| c.class()), Some(CellClass::Solid));
    assert_eq!(reg.get(9).map(|c| c.class()), Some(CellClass::Fluid));
    assert!(reg.get(0).is_none());
}

// -- registry.rs: clipped_cell_volume cell-merging floor ------------------------------

#[test]
fn clipped_cell_volume_floors_sliver_top_cell() {
    // A tiny cut top cell with cell-merging active is inflated to
    // `min_fraction · full_volume` (the floor arm of `clipped_cell_volume`).
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let geom = CubicalReggeGeometry::<2, f64>::uniform(1.0);
    let base = [1usize, 1usize];
    let top = LatticeCell::<2>::new(base, 0b11);
    let idx = lattice.cells(2).position(|c| c == top).unwrap();

    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(
        idx,
        CutCell::<2, f64>::cut(1.0, 0.01, [[1.0, 1.0], [1.0, 1.0]], Vec::new()),
    );
    let stab = reg.with_cell_merging(0.2);

    // Floored to 0.2 * full (full = 1.0 for unit geom).
    let v = stab.clipped_cell_volume(&geom, &lattice, &top);
    assert!(
        (v - 0.2).abs() < TOL,
        "sliver volume must floor to 0.2, got {v}"
    );

    // An unregistered fluid top cell falls through to the geometry fast path.
    let fluid_top = LatticeCell::<2>::new([0, 0], 0b11);
    let vf = stab.clipped_cell_volume(&geom, &lattice, &fluid_top);
    assert!((vf - 1.0).abs() < TOL);
}

// -- registry.rs: cut_face_constraints skips ------------------------------------------

#[test]
fn cut_face_constraints_skip_empty_fragment_cells() {
    // A `Cut` cell with no fragments contributes no rows (the `fragments
    // .is_empty()` skip).
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let top = LatticeCell::<2>::new([1, 1], 0b11);
    let idx = lattice.cells(2).position(|c| c == top).unwrap();
    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(
        idx,
        CutCell::<2, f64>::cut(1.0, 0.5, [[0.5, 0.5], [0.5, 0.5]], Vec::new()),
    );
    assert!(reg.cut_face_constraints(&lattice).is_empty());
}

#[test]
fn cut_face_constraints_skip_zero_area_fragment() {
    // A `Cut` cell whose only fragment has zero area gives zero total area and
    // is skipped (the `area_total <= 0` arm).
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let top = LatticeCell::<2>::new([1, 1], 0b11);
    let idx = lattice.cells(2).position(|c| c == top).unwrap();
    let frag = CutFaceFragment::<2, f64>::new(0.0, [1.0, 0.0], [0.5, 0.5], SourceGeometry::Plane);
    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(
        idx,
        CutCell::<2, f64>::cut(1.0, 0.5, [[0.5, 0.5], [0.5, 0.5]], vec![frag]),
    );
    assert!(reg.cut_face_constraints(&lattice).is_empty());
}

#[test]
fn cut_face_constraints_skip_zero_normal_fragment() {
    // A `Cut` cell whose fragment has positive area but a zero outward normal
    // gives a zero accumulated normal and is skipped (the `norm_sq <= 0` arm).
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let top = LatticeCell::<2>::new([1, 1], 0b11);
    let idx = lattice.cells(2).position(|c| c == top).unwrap();
    let frag = CutFaceFragment::<2, f64>::new(1.0, [0.0, 0.0], [0.5, 0.5], SourceGeometry::Plane);
    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(
        idx,
        CutCell::<2, f64>::cut(1.0, 0.5, [[0.5, 0.5], [0.5, 0.5]], vec![frag]),
    );
    assert!(reg.cut_face_constraints(&lattice).is_empty());
}

#[test]
fn cut_face_constraints_all_dry_apertures_use_uniform_reconstruction() {
    // A `Cut` cell whose apertures are all zero forces the aperture-weighted
    // reconstruction into its uniform fall-back (weight_sum == 0 arm). The
    // resulting rows must still be well-formed (NoPenetration + tangents).
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let top = LatticeCell::<2>::new([1, 1], 0b11);
    let idx = lattice.cells(2).position(|c| c == top).unwrap();
    let frag = CutFaceFragment::<2, f64>::new(1.0, [1.0, 0.0], [0.5, 0.5], SourceGeometry::Plane);
    let mut reg = CutCellRegistry::<2, f64>::new();
    reg.insert(
        idx,
        CutCell::<2, f64>::cut(1.0, 0.5, [[0.0, 0.0], [0.0, 0.0]], vec![frag]),
    );
    let rows = reg.cut_face_constraints(&lattice);
    // D = 2: one no-penetration row + one tangential row.
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .any(|r| r.kind() == CutConstraintKind::NoPenetration)
    );
    assert!(
        rows.iter()
            .any(|r| r.kind() == CutConstraintKind::Tangential)
    );
}

#[test]
fn cut_face_constraints_3d_yields_two_tangents() {
    // A 3D `Cut` cell with a fragment produces one no-penetration row and the
    // D - 1 = 2 orthonormal wall tangents (the D = 3 `wall_tangents` path,
    // including the second accepted tangent).
    let lattice = LatticeComplex::<3, f64>::cubic_torus(4);
    let top = LatticeCell::<3>::new([1, 1, 1], 0b111);
    let idx = lattice.cells(3).position(|c| c == top).unwrap();
    let frag = CutFaceFragment::<3, f64>::new(
        1.0,
        [0.0, 0.0, 1.0],
        [0.5, 0.5, 0.5],
        SourceGeometry::Plane,
    );
    let mut reg = CutCellRegistry::<3, f64>::new();
    reg.insert(
        idx,
        CutCell::<3, f64>::cut(1.0, 0.5, [[0.5, 0.5], [0.5, 0.5], [0.5, 0.5]], vec![frag]),
    );
    let rows = reg.cut_face_constraints(&lattice);
    // 1 no-penetration + 2 tangential = 3 rows.
    assert_eq!(rows.len(), 3);
    let n_tan = rows
        .iter()
        .filter(|r| r.kind() == CutConstraintKind::Tangential)
        .count();
    assert_eq!(n_tan, 2, "D=3 wall has two orthonormal tangents");
}
