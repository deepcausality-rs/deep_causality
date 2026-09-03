/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The tolerance family. Every member but the shot-noise one is a function of `R::epsilon()`, so
//! the one property that matters is that widening the scalar tightens each of them with no call
//! site changing; the shot-noise member reads the estimate and the shots instead.

use deep_causality_quantum::{DensityMatrix, Tolerance};

#[test]
fn test_the_shot_noise_width_exists_on_the_probability_axis_only() {
    let t = Tolerance::<f64>::shot_noise();
    assert!(t.shot_noise_width(0.5, 100).is_some());
    assert_eq!(t.shot_noise_width(0.0, 100), Some(0.0));
    assert_eq!(t.shot_noise_width(1.0, 100), Some(0.0));
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 1.1] {
        assert_eq!(t.shot_noise_width(bad, 100), None, "estimate {bad}");
    }
    // The other members have no width to give, whatever the estimate.
    assert_eq!(
        Tolerance::<f64>::validation().shot_noise_width(0.5, 100),
        None
    );
}

#[test]
fn test_the_numerical_rank_threshold_carries_the_dimension() {
    // The dimension enters as itself: a fallback that replaced it with one would be visible at
    // any dimension above one.
    let eps = f64::EPSILON;
    let t = Tolerance::<f64>::numerical_rank();
    assert_eq!(t.threshold(1, 1.0), Some(eps));
    assert_eq!(t.threshold(1024, 1.0), Some(eps * 1024.0));
    assert_eq!(
        t.threshold(1 << 40, 3.0),
        Some(eps * (1u64 << 40) as f64 * 3.0)
    );
}

#[test]
fn test_every_member_is_derived_from_epsilon() {
    let eps = f64::EPSILON;
    assert_eq!(Tolerance::<f64>::validation().epsilon(), eps);
    assert_eq!(
        Tolerance::<f64>::validation().threshold(4, 100.0),
        Some(eps.sqrt())
    );
    assert_eq!(
        Tolerance::<f64>::numerical_rank().threshold(4, 2.0),
        Some(eps * 4.0 * 2.0)
    );
    // State clamps the scale at one, as `DensityMatrix::with_tolerance` does.
    assert_eq!(
        Tolerance::<f64>::state().threshold(2, 0.5),
        Some(DensityMatrix::<f64>::default_tolerance())
    );
    assert_eq!(
        Tolerance::<f64>::state().threshold(2, 3.0),
        Some(DensityMatrix::<f64>::default_tolerance() * 3.0)
    );
}

#[test]
fn test_widening_the_scalar_tightens_every_member() {
    // f32 to f64 is the direction the design note's f64-to-Float106 example runs in; the
    // thresholds must move with the scalar and nothing else.
    let members_f32 = [
        Tolerance::<f32>::validation().threshold(4, 2.0).unwrap() as f64,
        Tolerance::<f32>::numerical_rank()
            .threshold(4, 2.0)
            .unwrap() as f64,
        Tolerance::<f32>::state().threshold(4, 2.0).unwrap() as f64,
    ];
    let members_f64 = [
        Tolerance::<f64>::validation().threshold(4, 2.0).unwrap(),
        Tolerance::<f64>::numerical_rank()
            .threshold(4, 2.0)
            .unwrap(),
        Tolerance::<f64>::state().threshold(4, 2.0).unwrap(),
    ];
    for (wide, narrow) in members_f64.iter().zip(members_f32) {
        assert!(
            wide < &narrow,
            "widening did not tighten: {wide} vs {narrow}"
        );
    }
}

#[test]
fn test_the_numerical_rank_member_stays_a_rank_cutoff() {
    // D·ε, not √ε: for f64 and D = 4 that is about 9e-16 against 1.5e-8. Using √ε here would
    // drop genuine range directions, which is what `range_projector`'s comment warns about.
    let rank = Tolerance::<f64>::numerical_rank()
        .threshold(4, 1.0)
        .unwrap();
    let validation = Tolerance::<f64>::validation().threshold(4, 1.0).unwrap();
    assert!(rank < validation * 1e-6);
}

#[test]
fn test_the_names_are_stable() {
    assert_eq!(Tolerance::<f64>::validation().name(), "validation");
    assert_eq!(Tolerance::<f64>::numerical_rank().name(), "numerical-rank");
    assert_eq!(Tolerance::<f64>::state().name(), "state");
}

#[cfg(feature = "qcm")]
mod commutator_member {
    use deep_causality_quantum::{CommutatorTolerance, Tolerance};

    #[test]
    fn test_the_commutator_member_answers_the_pair_form_only() {
        let q = Tolerance::<f64>::q_tol();
        assert_eq!(q.name(), "commutator");
        // The single-operator form has no answer for a pair policy.
        assert_eq!(q.threshold(4, 1.0), None);
        // The pair form delegates to the shipped policy exactly.
        let direct = CommutatorTolerance::<f64>::default().threshold(0, 1, 4, 1.5, 2.5);
        assert_eq!(q.commutator_threshold(0, 1, 4, 1.5, 2.5), Some(direct));
        // And the other members have no pair form.
        assert_eq!(
            Tolerance::<f64>::validation().commutator_threshold(0, 1, 4, 1.5, 2.5),
            None
        );
    }

    #[test]
    fn test_a_configured_commutator_policy_is_carried() {
        let policy = CommutatorTolerance::<f64>::default().with_safety_factor(16.0);
        let t = Tolerance::commutator(policy.clone());
        assert_eq!(
            t.commutator_threshold(0, 1, 2, 1.0, 1.0),
            Some(policy.threshold(0, 1, 2, 1.0, 1.0))
        );
        // Twice the safety factor is twice the threshold.
        let base = Tolerance::<f64>::q_tol()
            .commutator_threshold(0, 1, 2, 1.0, 1.0)
            .unwrap();
        assert!((t.commutator_threshold(0, 1, 2, 1.0, 1.0).unwrap() / base - 2.0).abs() < 1e-12);
    }
}
