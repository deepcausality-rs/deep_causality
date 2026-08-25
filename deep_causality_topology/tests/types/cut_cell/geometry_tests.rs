/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the closed-form clipping primitives behind `CutCell::from_box`:
//! the `[0, full]` clamp on the corner-sum volume, and the `[-r, r]` clamp in
//! the circular-quadrant antiderivative.

use deep_causality_topology::{CellClass, CutCell, Primitive};

#[test]
fn halfspace_far_beyond_the_cell_is_entirely_solid() {
    // The unit cube reaches at most `n · (1,1,1) = √3 ≈ 1.73` along the
    // normal, so a plane at offset 60000 leaves the whole cube on the solid
    // side and the clipped fluid volume is exactly zero. The corner sum
    // evaluates `relu(c − Σ aᵢlᵢ)³` at c ≈ 6e4, where four terms near 2e14
    // cancel down to a value near 1; the clamp is what keeps the answer inside
    // `[0, full]`.
    let inv = 1.0 / 3.0_f64.sqrt();
    let prim = Primitive::<3, f64>::Halfspace {
        normal: [inv, inv, inv],
        offset: 60_000.0,
    };
    let cell = CutCell::from_box(&prim, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).unwrap();

    assert_eq!(cell.class(), CellClass::Solid);
    assert_eq!(cell.full_volume(), 1.0);
    assert_eq!(cell.fluid_volume(), 0.0);
    assert_eq!(cell.volume_fraction(), 0.0);
}

#[test]
fn halfspace_far_behind_the_cell_is_entirely_fluid() {
    // Mirror of the above on the fluid side: with the plane at offset −60000
    // the cube lies wholly in `{ n · x ≥ offset }`, so the clipped fluid volume
    // is the full cell.
    let inv = 1.0 / 3.0_f64.sqrt();
    let prim = Primitive::<3, f64>::Halfspace {
        normal: [inv, inv, inv],
        offset: -60_000.0,
    };
    let cell = CutCell::from_box(&prim, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).unwrap();

    assert_eq!(cell.class(), CellClass::Fluid);
    assert_eq!(cell.fluid_volume(), 1.0);
}

#[test]
fn ball_with_negative_radius_leaves_the_cell_fluid() {
    // A ball of negative radius encloses no points, so the cell carries no
    // solid and stays fluid. The four-corner decomposition still evaluates the
    // circular-quadrant antiderivative, whose integration limits then sit
    // outside `[-r, r]` and are clamped to the endpoints.
    let prim = Primitive::<2, f64>::Ball {
        center: [0.0, 0.0],
        radius: -0.25,
    };
    let cell = CutCell::from_box(&prim, [0.5, 0.5], [1.5, 1.5]).unwrap();

    assert_eq!(cell.class(), CellClass::Fluid);
    assert_eq!(cell.fluid_volume(), 1.0);
    assert!(cell.fragments().is_empty());
}

// ---- degenerate and far-field geometry --------------------------------------------------------

/// A halfspace whose plane lies far outside the cell classifies by containment, not by quadrature.
///
/// The inclusion-exclusion sums `2^m` terms of size `c^m` and divides down to a result of order
/// the cell volume, so a large offset cancels the terms entirely. At an offset of 150000 on a unit
/// cell the rounding error came out negative, the clamp reported zero solid, and a cell lying
/// wholly inside the solid halfspace was classified `Fluid`.
#[test]
fn test_a_halfspace_far_outside_the_cell_classifies_by_containment() {
    let n = 1.0 / 3f64.sqrt();

    let inside = Primitive::<3, f64>::halfspace([n, n, n], 150_000.0);
    let cell = CutCell::from_box(&inside, [0.0; 3], [1.0; 3]).unwrap();
    assert_eq!(cell.class(), CellClass::Solid);
    assert_eq!(cell.fluid_volume(), 0.0);

    let outside = Primitive::<3, f64>::halfspace([n, n, n], -150_000.0);
    let cell = CutCell::from_box(&outside, [0.0; 3], [1.0; 3]).unwrap();
    assert_eq!(cell.class(), CellClass::Fluid);
    assert_eq!(cell.fluid_volume(), 1.0);
}

/// The short-circuit must not swallow a plane that genuinely cuts the cell.
#[test]
fn test_a_halfspace_that_cuts_the_cell_is_still_measured() {
    let n = 1.0 / 3f64.sqrt();
    let prim = Primitive::<3, f64>::halfspace([n, n, n], 0.8);
    let cell = CutCell::from_box(&prim, [0.0; 3], [1.0; 3]).unwrap();

    assert_eq!(cell.class(), CellClass::Cut);
    let fluid = cell.fluid_volume();
    assert!(
        fluid > 0.0 && fluid < 1.0,
        "a cutting plane must leave a partial volume, got {fluid}"
    );
}

/// A ball or cylinder of zero radius encloses nothing, so the cell is entirely fluid.
///
/// The circular-quadrant antiderivative evaluates `asin(u / r)`, which is `asin(0/0)` at zero
/// radius. The NaN reached the class comparisons, both of which are false against a NaN, and the
/// cell was recorded as `Cut` with NaN volumes.
#[test]
fn test_a_zero_radius_primitive_leaves_the_cell_fluid() {
    let ball = Primitive::<2, f64>::ball([0.5, 0.5], 0.0);
    let cell = CutCell::from_box(&ball, [0.0; 2], [1.0; 2]).unwrap();
    assert!(
        !cell.fluid_volume().is_nan(),
        "fluid volume must not be NaN"
    );
    assert_eq!(cell.class(), CellClass::Fluid);
    assert_eq!(cell.fluid_volume(), 1.0);

    let cyl = Primitive::<3, f64>::cylinder(2, [0.5, 0.5, 0.0], 0.0);
    let cell = CutCell::from_box(&cyl, [0.0; 3], [1.0; 3]).unwrap();
    assert!(
        !cell.fluid_volume().is_nan(),
        "fluid volume must not be NaN"
    );
    assert_eq!(cell.class(), CellClass::Fluid);
    assert_eq!(cell.fluid_volume(), 1.0);
}

/// A cell subtending a tiny angle still records its cut-face fragment.
///
/// The arc length came from a fixed 2048-point sweep of the full circle. A cell narrower than one
/// step could contain no sample and measured exactly zero, so a genuinely cut cell recorded no
/// fragment and the aperture-resolved no-slip stage lost that wetted surface. The cell below is
/// centred midway between two of the retired samples.
#[test]
fn test_a_cell_narrower_than_the_old_sampling_step_keeps_its_fragment() {
    let theta = 512.5 * 2.0 * std::f64::consts::PI / 2048.0;
    let (cx, cy) = (theta.cos(), theta.sin());
    let h = 5e-5;

    let unit_circle = Primitive::<2, f64>::ball([0.0, 0.0], 1.0);
    let cell = CutCell::from_box(&unit_circle, [cx - h, cy - h], [cx + h, cy + h]).unwrap();

    assert_eq!(cell.class(), CellClass::Cut);
    assert_eq!(
        cell.fragments().len(),
        1,
        "the circle passes through this cell, so it has a cut-face fragment"
    );
}
