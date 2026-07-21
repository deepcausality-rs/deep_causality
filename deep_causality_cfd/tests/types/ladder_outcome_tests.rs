/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for [`LadderOutcome`] — the refinement-ladder verdict that reports non-convergence as a
//! result rather than folding it into a pass or a bare failure.

use deep_causality_cfd::LadderOutcome;

#[test]
fn test_settling_sequence_converges() {
    // Successive differences 1.0, 0.1, 0.01 — the finest is inside the tolerance.
    let values = [10.0, 11.0, 11.1, 11.11];
    let out = LadderOutcome::judge(&values, 0.05);
    assert!(out.is_converged());
    assert!(!out.is_not_converging());
    let d = out.final_delta().expect("converged carries a delta");
    assert!((d - 0.01).abs() < 1e-12, "final delta was {d}");
}

#[test]
fn test_shrinking_but_outside_tolerance_is_not_converging() {
    // Still refining: the delta shrank but has not reached the bound.
    let values = [10.0, 11.0, 11.5];
    let out = LadderOutcome::judge(&values, 0.01);
    assert!(out.is_not_converging());
    assert!(!out.is_converged());
    match out {
        LadderOutcome::NotConverging { detail, .. } => {
            assert!(detail.contains("shrinking"), "detail was: {detail}");
            assert!(detail.contains("refine further"), "detail was: {detail}");
        }
        other => panic!("expected NotConverging, got {other:?}"),
    }
}

#[test]
fn test_reversing_sequence_reports_no_demonstrated_limit() {
    // The QTT cylinder's measured eta ladder: 17.39 -> 24.02 -> 26.25 -> 23.76 -> 21.40. It rises
    // then falls, so its FINAL delta (2.36) is smaller than the previous one (2.49) even though
    // the observable is approaching nothing. Judging on delta shrinkage alone would call this
    // "shrinking, refine further" and understate it; the reversal is the signal that no limit
    // exists for this ladder to reach.
    let values = [17.39, 24.02, 26.25, 23.76, 21.40];
    let out = LadderOutcome::judge(&values, 0.01);
    assert!(out.is_not_converging());
    match out {
        LadderOutcome::NotConverging { detail, .. } => {
            assert!(detail.contains("not settling"), "detail was: {detail}");
            assert!(detail.contains("non-monotone"), "detail was: {detail}");
            assert!(
                detail.contains("no limit is demonstrated"),
                "detail was: {detail}"
            );
        }
        other => panic!("expected NotConverging, got {other:?}"),
    }
}

#[test]
fn test_monotone_but_growing_sequence_reports_no_demonstrated_limit() {
    // Diverging outright, without any reversal — the other route to "not settling".
    let values = [1.0, 2.0, 4.0, 8.0];
    let out = LadderOutcome::judge(&values, 0.01);
    assert!(out.is_not_converging());
    match out {
        LadderOutcome::NotConverging { detail, .. } => {
            assert!(detail.contains("did not shrink"), "detail was: {detail}");
            assert!(!detail.contains("non-monotone"), "detail was: {detail}");
        }
        other => panic!("expected NotConverging, got {other:?}"),
    }
}

#[test]
fn test_reversal_within_tolerance_still_converges() {
    // Round-off-scale sign flips near a limit must not be reported as non-convergence: the
    // tolerance check runs first.
    let values = [5.0, 5.000_002, 4.999_999];
    let out = LadderOutcome::judge(&values, 1.0e-4);
    assert!(out.is_converged(), "got {out:?}");
}

#[test]
fn test_bond_ladder_with_generous_bound_still_converges() {
    // The committed bond ladder: 24.0543 -> 23.7649 -> 23.7577 -> 23.7577.
    let values = [24.0543, 23.7649, 23.7577, 23.7577];
    assert!(LadderOutcome::judge(&values, 1.0e-6).is_converged());
}

#[test]
fn test_bond_ladder_fails_a_bound_tightened_below_the_measured_delta() {
    // Tightening the bound below the measured difference must be able to fail — this is the
    // property the audit found missing when CONVERGENCE_BOUND sat eleven orders above the delta.
    let values = [24.0543, 23.7649, 23.7577, 23.7576];
    let out = LadderOutcome::judge(&values, 1.0e-12);
    assert!(out.is_not_converging(), "got {out:?}");
}

#[test]
fn test_two_rungs_are_indeterminate() {
    // One difference is not a trend.
    let out = LadderOutcome::judge(&[1.0, 2.0], 0.1);
    assert!(matches!(out, LadderOutcome::Indeterminate { .. }));
    assert!(!out.is_converged());
    assert!(!out.is_not_converging());
    assert_eq!(out.final_delta(), None);
}

#[test]
fn test_empty_and_single_rung_are_indeterminate() {
    assert!(matches!(
        LadderOutcome::judge(&[], 0.1),
        LadderOutcome::Indeterminate { .. }
    ));
    assert!(matches!(
        LadderOutcome::judge(&[1.0], 0.1),
        LadderOutcome::Indeterminate { .. }
    ));
}

#[test]
fn test_nan_does_not_read_as_convergence() {
    // Every comparison against NaN is false, so an unguarded implementation would report
    // convergence for a diverged run. It must be Indeterminate instead.
    let out = LadderOutcome::judge(&[1.0, 2.0, f64::NAN], 0.1);
    assert!(matches!(out, LadderOutcome::Indeterminate { .. }));
    assert!(!out.is_converged());
}

#[test]
fn test_infinite_value_is_indeterminate() {
    let out = LadderOutcome::judge(&[1.0, 2.0, f64::INFINITY], 0.1);
    assert!(matches!(out, LadderOutcome::Indeterminate { .. }));
}

#[test]
fn test_non_positive_or_non_finite_tolerance_is_indeterminate() {
    let values = [10.0, 11.0, 11.1];
    assert!(matches!(
        LadderOutcome::judge(&values, 0.0),
        LadderOutcome::Indeterminate { .. }
    ));
    assert!(matches!(
        LadderOutcome::judge(&values, -1.0),
        LadderOutcome::Indeterminate { .. }
    ));
    assert!(matches!(
        LadderOutcome::judge(&values, f64::NAN),
        LadderOutcome::Indeterminate { .. }
    ));
}

#[test]
fn test_display_marks_non_convergence_loudly() {
    let converged = LadderOutcome::judge(&[10.0, 11.0, 11.001], 0.01);
    assert!(format!("{converged}").starts_with("converged"));

    let drifting = LadderOutcome::judge(&[17.39, 24.02, 26.25, 23.76, 21.40], 0.01);
    let rendered = format!("{drifting}");
    assert!(
        rendered.contains("NOT CONVERGING"),
        "rendered as: {rendered}"
    );

    let short = LadderOutcome::judge(&[1.0], 0.1);
    assert!(format!("{short}").starts_with("indeterminate"));
}

#[test]
fn test_exactly_at_tolerance_converges() {
    // Boundary: the comparison is inclusive.
    let out = LadderOutcome::judge(&[1.0, 2.0, 2.5], 0.5);
    assert!(out.is_converged(), "got {out:?}");
}

#[test]
fn test_flat_sequence_converges() {
    // A fully saturated ladder: zero difference at the finest pair.
    let out = LadderOutcome::judge(&[5.0, 5.0, 5.0], 1e-15);
    assert!(out.is_converged());
    assert_eq!(out.final_delta(), Some(0.0));
}
