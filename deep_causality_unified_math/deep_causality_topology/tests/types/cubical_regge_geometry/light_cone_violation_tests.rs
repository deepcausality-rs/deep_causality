/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `LightConeViolation` produced by the Lorentzian constructor on
//! `CubicalReggeGeometry<D, R, Lorentzian>` — Phase R5.5.
//!
//! Sylvester's-criterion check on axis-aligned cubical metrics reduces to
//! "exactly one timelike axis" (one negative eigenvalue, D−1 positive). The
//! constructor rejects:
//!
//! - **0 timelike axes** with `LightConeViolation::AllSpacelike`.
//! - **≥ 2 timelike axes** with `LightConeViolation::CellSignature` carrying
//!   the synthesized per-axis diagonal-sign eigenvalue pattern.

use deep_causality_topology::{CubicalReggeGeometry, LightConeViolation};

#[test]
fn all_spacelike_pattern_rejected_2d() {
    let err = CubicalReggeGeometry::<2, f64>::unit()
        .with_timelike_axes([false, false])
        .expect_err("all-spacelike must error");
    assert!(matches!(err, LightConeViolation::AllSpacelike));
}

#[test]
fn all_spacelike_pattern_rejected_4d() {
    let err = CubicalReggeGeometry::<4, f64>::unit()
        .with_timelike_axes([false, false, false, false])
        .expect_err("all-spacelike must error");
    assert!(matches!(err, LightConeViolation::AllSpacelike));
}

#[test]
fn two_timelike_axes_rejected_3d() {
    // 3D with axes 0 and 1 both timelike — signature (1, 2, 0), not Lorentzian.
    let err = CubicalReggeGeometry::<3, f64>::unit()
        .with_timelike_axes([true, true, false])
        .expect_err("two timelike axes must error");
    match err {
        LightConeViolation::CellSignature {
            cell_id,
            eigenvalues,
        } => {
            assert_eq!(cell_id, 0);
            assert_eq!(eigenvalues, vec![-1.0, -1.0, 1.0]);
        }
        other => panic!("expected CellSignature, got {other:?}"),
    }
}

#[test]
fn three_timelike_axes_rejected_4d() {
    let err = CubicalReggeGeometry::<4, f64>::unit()
        .with_timelike_axes([true, true, true, false])
        .expect_err("three timelike axes must error");
    assert!(matches!(err, LightConeViolation::CellSignature { .. }));
}

#[test]
fn exactly_one_timelike_axis_accepted_at_every_position() {
    // All D positions of the single timelike axis must produce a valid Lorentzian.
    for pos in 0..3 {
        let mut pattern = [false; 3];
        pattern[pos] = true;
        let g = CubicalReggeGeometry::<3, f64>::unit().with_timelike_axes(pattern);
        assert!(g.is_ok(), "axis {pos} alone timelike must succeed");
        assert!(g.unwrap().is_lorentzian());
    }
}

#[test]
fn error_displays_human_readable_message() {
    let err: LightConeViolation<f64> = LightConeViolation::AllSpacelike;
    let msg = format!("{err}");
    assert!(msg.contains("Lorentzian"));
    assert!(msg.contains("timelike"));
}

#[test]
fn error_cell_signature_displays_with_eigenvalues() {
    let err: LightConeViolation<f64> = LightConeViolation::CellSignature {
        cell_id: 0,
        eigenvalues: vec![-1.0, -1.0, 1.0],
    };
    let msg = format!("{err}");
    assert!(msg.contains("Light-cone violation"));
    assert!(msg.contains("eigenvalues"));
}
