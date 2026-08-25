/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact-intersection tests (A4, A7): clipped volume and apertures of cube ∩ {plane, disk,
//! cylinder} against the closed-form **measures**, at f64 and Float106. Cochain discipline:
//! every comparison is measure-vs-measure, never against a pointwise value (the lesson the
//! `graded-metrics` study established).

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_topology::{CellClass, CutCell, Primitive, SourceGeometry};

fn r<R: RealField + FromPrimitive>(x: f64) -> R {
    R::from_f64(x).expect("f64 literal representable in R")
}

fn close<R: RealField + FromPrimitive>(a: R, b: R, tol: f64) -> bool {
    (a - b).abs() < r::<R>(tol)
}

// -- Half-space (plane), all dimensions -------------------------------------------------

fn axis_aligned_plane_2d<R: RealField + FromPrimitive>() {
    // Solid { x ≤ 0.3 } in the unit cell ⇒ fluid measure 0.7.
    let prim = Primitive::<2, R>::halfspace([r::<R>(1.0), r::<R>(0.0)], r::<R>(0.3));
    let cell = CutCell::from_box(
        &prim,
        [r::<R>(0.0), r::<R>(0.0)],
        [r::<R>(1.0), r::<R>(1.0)],
    )
    .unwrap();
    assert_eq!(cell.class(), CellClass::Cut);
    assert!(close(cell.fluid_volume(), r::<R>(0.7), 1e-12));
    assert!(close(cell.full_volume(), r::<R>(1.0), 1e-12));
    // Apertures: low-x face fully dry, high-x face fully wet, y-faces 70% wet.
    assert!(close(cell.face_aperture(0, 0).unwrap(), r::<R>(0.0), 1e-12));
    assert!(close(cell.face_aperture(0, 1).unwrap(), r::<R>(1.0), 1e-12));
    assert!(close(cell.face_aperture(1, 0).unwrap(), r::<R>(0.7), 1e-12));
    assert!(close(cell.face_aperture(1, 1).unwrap(), r::<R>(0.7), 1e-12));
    // Fragment: the planar cross-section measure is the full unit edge, normal into fluid.
    assert_eq!(cell.fragments().len(), 1);
    let f = &cell.fragments()[0];
    assert!(close(f.area(), r::<R>(1.0), 1e-12));
    assert_eq!(f.source(), SourceGeometry::Plane);
    assert!(close(f.outward_normal()[0], r::<R>(1.0), 1e-12));
    assert!(close(f.outward_normal()[1], r::<R>(0.0), 1e-12));
}

fn diagonal_plane_2d<R: RealField + FromPrimitive>() {
    // Solid { x + y ≤ 1 } in the unit cell ⇒ a triangle of area 1/2.
    let prim = Primitive::<2, R>::halfspace([r::<R>(1.0), r::<R>(1.0)], r::<R>(1.0));
    let cell = CutCell::from_box(
        &prim,
        [r::<R>(0.0), r::<R>(0.0)],
        [r::<R>(1.0), r::<R>(1.0)],
    )
    .unwrap();
    assert!(close(cell.fluid_volume(), r::<R>(0.5), 1e-12));
    // The cross-section is the cube diagonal: length sqrt(2).
    assert!(close(cell.fragments()[0].area(), r::<R>(2.0).sqrt(), 1e-12));
}

fn axis_aligned_plane_3d<R: RealField + FromPrimitive>() {
    // Solid { z ≤ 0.25 } ⇒ fluid measure 0.75; cross-section is a unit square.
    let prim = Primitive::<3, R>::halfspace([r::<R>(0.0), r::<R>(0.0), r::<R>(1.0)], r::<R>(0.25));
    let cell = CutCell::from_box(
        &prim,
        [r::<R>(0.0), r::<R>(0.0), r::<R>(0.0)],
        [r::<R>(1.0), r::<R>(1.0), r::<R>(1.0)],
    )
    .unwrap();
    assert!(close(cell.fluid_volume(), r::<R>(0.75), 1e-12));
    assert!(close(cell.face_aperture(2, 0).unwrap(), r::<R>(0.0), 1e-12));
    assert!(close(cell.face_aperture(2, 1).unwrap(), r::<R>(1.0), 1e-12));
    assert!(close(
        cell.face_aperture(0, 0).unwrap(),
        r::<R>(0.75),
        1e-12
    ));
    assert!(close(cell.fragments()[0].area(), r::<R>(1.0), 1e-12));
}

fn plane_classifies_full_cells<R: RealField + FromPrimitive>() {
    // Plane entirely outside the cell on the fluid side ⇒ Fluid.
    let prim = Primitive::<2, R>::halfspace([r::<R>(1.0), r::<R>(0.0)], r::<R>(-1.0));
    let cell = CutCell::from_box(
        &prim,
        [r::<R>(0.0), r::<R>(0.0)],
        [r::<R>(1.0), r::<R>(1.0)],
    )
    .unwrap();
    assert_eq!(cell.class(), CellClass::Fluid);
    // Plane entirely outside on the solid side ⇒ Solid.
    let prim = Primitive::<2, R>::halfspace([r::<R>(1.0), r::<R>(0.0)], r::<R>(2.0));
    let cell = CutCell::from_box(
        &prim,
        [r::<R>(0.0), r::<R>(0.0)],
        [r::<R>(1.0), r::<R>(1.0)],
    )
    .unwrap();
    assert_eq!(cell.class(), CellClass::Solid);
}

// -- Disk (2D ball) ---------------------------------------------------------------------

fn quarter_disk_2d<R: RealField + FromPrimitive>() {
    // Unit disk centred at the cell corner ⇒ the solid is a quarter disk, area π/4.
    let prim = Primitive::<2, R>::ball([r::<R>(0.0), r::<R>(0.0)], r::<R>(1.0));
    let cell = CutCell::from_box(
        &prim,
        [r::<R>(0.0), r::<R>(0.0)],
        [r::<R>(1.0), r::<R>(1.0)],
    )
    .unwrap();
    assert_eq!(cell.class(), CellClass::Cut);
    let quarter = R::pi() / r::<R>(4.0);
    assert!(close(cell.fluid_volume(), r::<R>(1.0) - quarter, 1e-12));
    // The x=0 / y=0 faces lie on the disk diameter ⇒ fully dry; the far faces are wet.
    assert!(close(cell.face_aperture(0, 0).unwrap(), r::<R>(0.0), 1e-12));
    assert!(close(cell.face_aperture(0, 1).unwrap(), r::<R>(1.0), 1e-12));
    assert!(close(cell.face_aperture(1, 0).unwrap(), r::<R>(0.0), 1e-12));
    assert_eq!(cell.fragments()[0].source(), SourceGeometry::Sphere);
}

fn half_disk_strip_2d<R: RealField + FromPrimitive>() {
    // Disk radius 1 centred at the mid-bottom edge of a 2x1 cell ⇒ a half disk, area π/2.
    let prim = Primitive::<2, R>::ball([r::<R>(1.0), r::<R>(0.0)], r::<R>(1.0));
    let cell = CutCell::from_box(
        &prim,
        [r::<R>(0.0), r::<R>(0.0)],
        [r::<R>(2.0), r::<R>(1.0)],
    )
    .unwrap();
    let half = R::pi() / r::<R>(2.0);
    let full = r::<R>(2.0);
    assert!(close(cell.fluid_volume(), full - half, 1e-12));
}

// -- Cylinder (3D) ----------------------------------------------------------------------

fn quarter_cylinder_3d<R: RealField + FromPrimitive>() {
    // Cylinder along z, radius 1, centre line through the cell's z-edge corner ⇒ the cross
    // section is a quarter disk (π/4) extruded over unit height ⇒ solid measure π/4.
    let prim = Primitive::<3, R>::cylinder(2, [r::<R>(0.0), r::<R>(0.0), r::<R>(0.0)], r::<R>(1.0));
    let cell = CutCell::from_box(
        &prim,
        [r::<R>(0.0), r::<R>(0.0), r::<R>(0.0)],
        [r::<R>(1.0), r::<R>(1.0), r::<R>(1.0)],
    )
    .unwrap();
    assert_eq!(cell.class(), CellClass::Cut);
    let quarter = R::pi() / r::<R>(4.0);
    assert!(close(cell.fluid_volume(), r::<R>(1.0) - quarter, 1e-12));
    // Faces perpendicular to the cylinder axis carry the full cross-section aperture.
    let cross_fluid = (r::<R>(1.0) - quarter) / r::<R>(1.0);
    assert!(close(cell.face_aperture(2, 0).unwrap(), cross_fluid, 1e-12));
    assert!(close(cell.face_aperture(2, 1).unwrap(), cross_fluid, 1e-12));
    assert_eq!(cell.fragments()[0].source(), SourceGeometry::Cylinder);
}

// -- Unsupported combinations error rather than silently mis-clip -----------------------

#[test]
fn ball_in_3d_is_rejected() {
    let prim = Primitive::<3, f64>::ball([0.0, 0.0, 0.0], 1.0);
    let res = CutCell::from_box(&prim, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    assert!(res.is_err());
}

#[test]
fn cylinder_in_2d_is_rejected() {
    let prim = Primitive::<2, f64>::cylinder(0, [0.0, 0.0], 1.0);
    let res = CutCell::from_box(&prim, [0.0, 0.0], [1.0, 1.0]);
    assert!(res.is_err());
}

// -- f64 and Float106 instantiations ----------------------------------------------------

#[test]
fn exact_intersections_f64() {
    axis_aligned_plane_2d::<f64>();
    diagonal_plane_2d::<f64>();
    axis_aligned_plane_3d::<f64>();
    plane_classifies_full_cells::<f64>();
    quarter_disk_2d::<f64>();
    half_disk_strip_2d::<f64>();
    quarter_cylinder_3d::<f64>();
}

#[test]
fn exact_intersections_f106() {
    axis_aligned_plane_2d::<Float106>();
    diagonal_plane_2d::<Float106>();
    axis_aligned_plane_3d::<Float106>();
    plane_classifies_full_cells::<Float106>();
    quarter_disk_2d::<Float106>();
    half_disk_strip_2d::<Float106>();
    quarter_cylinder_3d::<Float106>();
}

// -- Degenerate cell shapes and sub-quadrature cuts ---------------------------------------

#[test]
fn plane_through_a_highly_elongated_cell_keeps_exact_measures() {
    // Cell 1e13 x 1 x 1 cut in half by { x ≤ 5e12 }: the clipped fluid measure
    // is exactly half the cell, the two x-faces are fully dry and fully wet,
    // and the four faces parallel to x are half wet. The cut cross-section
    // (area 1) is fourteen orders of magnitude below the cell volume, which is
    // the scale the classification tolerance is drawn from.
    let prim = Primitive::<3, f64>::halfspace([1.0, 0.0, 0.0], 5e12);
    let cell = CutCell::from_box(&prim, [0.0, 0.0, 0.0], [1e13, 1.0, 1.0]).unwrap();

    assert_eq!(cell.class(), CellClass::Cut);
    assert_eq!(cell.full_volume(), 1e13);
    assert_eq!(cell.fluid_volume(), 5e12);
    assert_eq!(cell.volume_fraction(), 0.5);
    assert_eq!(cell.face_aperture(0, 0).unwrap(), 0.0);
    assert_eq!(cell.face_aperture(0, 1).unwrap(), 1.0);
    for axis in 1..3 {
        for side in 0..2 {
            assert_eq!(cell.face_aperture(axis, side).unwrap(), 0.5);
        }
    }
}

/// A cell of side `h` centred on the unit circle, midway between two samples of
/// the arc quadrature, at polar angle `(512 + 1/2) · 2π / 2048`.
fn micro_cell_on_the_unit_circle(h: f64) -> ([f64; 2], [f64; 2]) {
    let theta = 512.5 * (2.0 * std::f64::consts::PI / 2048.0);
    let (cx, cy) = (theta.cos(), theta.sin());
    ([cx - h / 2.0, cy - h / 2.0], [cx + h / 2.0, cy + h / 2.0])
}

#[test]
fn disk_cut_far_finer_than_the_arc_quadrature_still_clips_exactly() {
    // A 1e-4 cell straddling the unit circle: the circle crosses the cell
    // centre, so a straight chord would halve it and the curvature correction
    // over 1e-4 of arc is below 1e-5 of the cell. The clipped measure comes
    // from the closed-form rectangle-disk area and is exact regardless of the
    // cell's size relative to the circle.
    let (lo, hi) = micro_cell_on_the_unit_circle(1e-4);
    let prim = Primitive::<2, f64>::ball([0.0, 0.0], 1.0);
    let cell = CutCell::from_box(&prim, lo, hi).unwrap();

    assert_eq!(cell.class(), CellClass::Cut);
    let frac = cell.volume_fraction();
    assert!(
        (frac - 0.5).abs() < 1e-3,
        "the circle halves the cell, got fluid fraction {frac}"
    );
}

#[test]
fn cylinder_cut_far_finer_than_the_arc_quadrature_still_clips_exactly() {
    // The 3D reading of the same configuration: a cylinder of unit radius along
    // axis 2 through a cell whose cross-section is 1e-4 on a side. The clipped
    // volume is the cross-section measure times the cell's length along the
    // cylinder axis, so the fluid fraction is again one half.
    let (lo2, hi2) = micro_cell_on_the_unit_circle(1e-4);
    let prim = Primitive::<3, f64>::cylinder(2, [0.0, 0.0, 0.0], 1.0);
    let cell = CutCell::from_box(&prim, [lo2[0], lo2[1], 0.0], [hi2[0], hi2[1], 1.0]).unwrap();

    assert_eq!(cell.class(), CellClass::Cut);
    let frac = cell.volume_fraction();
    assert!(
        (frac - 0.5).abs() < 1e-3,
        "the cylinder halves the cell, got fluid fraction {frac}"
    );
}

// ---- the fragment tolerance carries the right dimensions --------------------------------------

/// A highly elongated cell cut in half still records its cut-face fragment.
///
/// The fragment threshold compared a cross-section **area** against a **volume**-scaled tolerance,
/// `cell_volume * 1e-12`. On a `1e13 x 1 x 1` cell that tolerance is 10 while the cut area is 1,
/// so a cell cut exactly in half recorded no fragment at all. The tolerance now scales with the
/// quantity it gates: the smallest product of `D−1` of the cell's extents, which is the smallest
/// 2-face here.
#[test]
fn test_a_plane_through_a_highly_elongated_cell_keeps_its_fragment() {
    let prim = Primitive::<3, f64>::halfspace([1.0, 0.0, 0.0], 5e12);
    let cell = CutCell::from_box(&prim, [0.0; 3], [1e13, 1.0, 1.0]).unwrap();

    assert_eq!(cell.class(), CellClass::Cut);
    assert_eq!(
        cell.fragments().len(),
        1,
        "a cell cut through its middle has a cut face whatever its aspect ratio"
    );

    let half = 1e13 * 0.5;
    assert!(
        (cell.fluid_volume() - half).abs() < half * 1e-9,
        "half the cell is fluid: expected {half}, got {}",
        cell.fluid_volume()
    );
}

/// The same cell at a normal aspect ratio, so the test above is about the tolerance and not
/// about the cut.
#[test]
fn test_a_plane_through_a_unit_cell_keeps_its_fragment() {
    let prim = Primitive::<3, f64>::halfspace([1.0, 0.0, 0.0], 0.5);
    let cell = CutCell::from_box(&prim, [0.0; 3], [1.0; 3]).unwrap();

    assert_eq!(cell.class(), CellClass::Cut);
    assert_eq!(cell.fragments().len(), 1);
    assert!((cell.fluid_volume() - 0.5).abs() < 1e-12);
}

/// The 2D reading of the same cut: an elongated planar cell halved by a vertical line.
///
/// The cut-face measure of a `D`-dimensional cell is a `(D−1)`-measure, so at `D = 2` it is a
/// length. Gating it against a tolerance drawn from the cell's smallest 2-face compares a length
/// against an area: on a `1e13 x 1` cell that tolerance is `1e13 * 1 * 1e-12 = 10`, while the cut
/// runs the full height of the cell and measures `1`. The fragment survives because the tolerance
/// is built from the smallest product of `D−1` extents, which is `min(1e13, 1) * 1e-12 = 1e-12`
/// here.
#[test]
fn test_a_line_through_a_highly_elongated_2d_cell_keeps_its_fragment() {
    let prim = Primitive::<2, f64>::halfspace([1.0, 0.0], 5e12);
    let cell = CutCell::from_box(&prim, [0.0; 2], [1e13, 1.0]).unwrap();

    assert_eq!(cell.class(), CellClass::Cut);
    assert_eq!(
        cell.fragments().len(),
        1,
        "a cell cut through its middle has a cut face whatever its aspect ratio"
    );

    let f = &cell.fragments()[0];
    assert_eq!(f.source(), SourceGeometry::Plane);
    assert!(
        (f.area() - 1.0).abs() < 1e-9,
        "the cut spans the cell's height: expected 1, got {}",
        f.area()
    );
    // The centroid sits on the cut line, at the cell centre along the uncut axis.
    assert!((f.centroid()[0] - 5e12).abs() < 5e12 * 1e-12);
    assert!((f.centroid()[1] - 0.5).abs() < 1e-12);

    let half = 1e13 * 0.5;
    assert!(
        (cell.fluid_volume() - half).abs() < half * 1e-9,
        "half the cell is fluid: expected {half}, got {}",
        cell.fluid_volume()
    );
}
