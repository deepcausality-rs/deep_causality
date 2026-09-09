/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Spec`, the real-valued threshold a read-out is judged against.
//!
//! # Risk, and the input that separates it
//!
//! `judge` is a two-arm dispatch: `AtLeast` must reach `at_least` and `AtMost` must reach
//! `at_most`. The arms are interchangeable at any estimate that satisfies both bounds or
//! neither — an estimate exactly at the threshold satisfies both, and the file was covered
//! only through the pipeline, where no case pinned which arm ran.
//!
//! The separating input is therefore an estimate on ONE side of the threshold, where the two
//! specs disagree: `AtLeast` accepts and `AtMost` rejects, or the reverse.

use deep_causality_quantum::{ShotEstimate, Spec};

/// A high-shot estimate, so the shot-noise band is narrow enough that a 0.2 gap is decisive
/// rather than absorbed by the standard error.
fn estimate(p: f64) -> ShotEstimate<f64> {
    ShotEstimate::from_probability(p, 100_000).expect("a probability with a shot budget")
}

#[test]
fn test_the_two_arms_are_not_interchangeable() {
    // p = 0.7 is above 0.5 and below 0.9. Each spec accepts on one side and rejects on the
    // other, so swapping the match arms flips all four verdicts.
    let p = estimate(0.7);

    assert!(
        Spec::at_least(0.5).judge(&p).accepted(),
        "0.7 reaches 0.5 from below"
    );
    assert!(
        !Spec::at_least(0.9).judge(&p).accepted(),
        "0.7 does not reach 0.9"
    );
    assert!(
        Spec::at_most(0.9).judge(&p).accepted(),
        "0.7 stays at or below 0.9"
    );
    assert!(
        !Spec::at_most(0.5).judge(&p).accepted(),
        "0.7 does not stay below 0.5"
    );
}

#[test]
fn test_judge_agrees_with_the_estimate_it_delegates_to() {
    // The dispatch carries the threshold through unchanged: same verdict, same margin as
    // calling the estimate directly. A spec that dropped or negated its value would differ.
    let p = estimate(0.7);
    for v in [0.1_f64, 0.5, 0.7, 0.9] {
        let via_spec = Spec::at_least(v).judge(&p);
        let direct = p.at_least(v);
        assert_eq!(via_spec.accepted(), direct.accepted(), "at_least({v})");
        assert_eq!(
            via_spec.worst_margin(),
            direct.worst_margin(),
            "at_least({v})"
        );

        let via_spec = Spec::at_most(v).judge(&p);
        let direct = p.at_most(v);
        assert_eq!(via_spec.accepted(), direct.accepted(), "at_most({v})");
        assert_eq!(
            via_spec.worst_margin(),
            direct.worst_margin(),
            "at_most({v})"
        );
    }
}

#[test]
fn test_the_constructors_build_the_variant_they_name() {
    // `at_least` must not build `AtMost`. Checked on the variant itself, so it holds even
    // where the two judgements would agree.
    assert_eq!(Spec::at_least(0.25_f64), Spec::AtLeast(0.25));
    assert_eq!(Spec::at_most(0.25_f64), Spec::AtMost(0.25));
    assert_ne!(Spec::at_least(0.25_f64), Spec::at_most(0.25));
}

#[test]
fn test_a_threshold_at_the_estimate_is_accepted_by_both_bounds() {
    // The boundary itself: `>=` and `<=` are both satisfied, which is why this input cannot
    // separate the arms and the test above uses an off-threshold estimate instead.
    let p = estimate(0.6);
    assert!(Spec::at_least(0.6).judge(&p).accepted());
    assert!(Spec::at_most(0.6).judge(&p).accepted());
}
