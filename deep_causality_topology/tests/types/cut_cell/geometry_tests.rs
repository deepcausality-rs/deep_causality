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
