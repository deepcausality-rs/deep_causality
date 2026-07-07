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
