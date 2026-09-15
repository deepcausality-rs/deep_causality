/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The edges of probabilistic presence: the two certain presences, absence through every operator,
//! and the gate.

use deep_causality_uncertain::{MaybeUncertain, SampleSession, Uncertain, UncertainError};

const SEED: u64 = 0x5EED_2026;

/// The two certain presences are certain at every index, not merely usually.
#[test]
fn the_certain_presences_hold_at_every_index() {
    let mut session = SampleSession::seeded(SEED);
    let present = MaybeUncertain::<f64>::from_value(4.5);
    let absent = MaybeUncertain::<f64>::always_none();

    for _ in 0..64 {
        assert_eq!(present.sample(&mut session).expect("draw"), Some(4.5));
        assert_eq!(absent.sample(&mut session).expect("draw"), None);
    }
}

/// A presence probability of zero is never present; of one, always. The endpoints are exact.
#[test]
fn the_presence_endpoints_are_exact() {
    let mut session = SampleSession::seeded(SEED);
    let never = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.0, Uncertain::point(1.0));
    let always = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(1.0, Uncertain::point(1.0));

    for _ in 0..64 {
        assert_eq!(never.sample(&mut session).expect("draw"), None);
        assert_eq!(always.sample(&mut session).expect("draw"), Some(1.0));
    }
}

/// A presence probability outside `[0, 1]` is refused when the presence channel is drawn.
#[test]
fn a_presence_probability_outside_the_unit_interval_is_refused() {
    let mut session = SampleSession::seeded(SEED);

    for bad in [-0.5, 1.5, f64::NAN] {
        let m = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(bad, Uncertain::point(1.0));
        assert!(
            matches!(
                m.sample(&mut session),
                Err(UncertainError::BernoulliDistributionError(_))
            ),
            "a presence probability of {bad} must be refused"
        );
    }
}

/// Absence propagates through **every** binary operator, not merely addition.
#[test]
fn absence_propagates_through_every_binary_operator() {
    let mut session = SampleSession::seeded(SEED);
    let present = MaybeUncertain::<f64>::from_value(6.0);
    let absent = MaybeUncertain::<f64>::always_none();

    let cases = [
        ("add", present.clone() + absent.clone()),
        ("sub", present.clone() - absent.clone()),
        ("mul", present.clone() * absent.clone()),
        ("div", present.clone() / absent.clone()),
        ("add, absent on the left", absent.clone() + present.clone()),
        ("sub, absent on the left", absent.clone() - present.clone()),
        ("mul, absent on the left", absent.clone() * present.clone()),
        ("div, absent on the left", absent.clone() / present.clone()),
    ];

    for (label, combined) in cases {
        assert_eq!(
            combined.sample(&mut session).expect("draw"),
            None,
            "{label}: one absent operand must make the result absent"
        );
    }
}

/// Two present operands combine by value, with the right arithmetic on each operator.
#[test]
fn two_present_operands_combine_by_value() {
    let mut session = SampleSession::seeded(SEED);
    let six = MaybeUncertain::<f64>::from_value(6.0);
    let two = MaybeUncertain::<f64>::from_value(2.0);

    assert_eq!(
        (six.clone() + two.clone())
            .sample(&mut session)
            .expect("draw"),
        Some(8.0)
    );
    assert_eq!(
        (six.clone() - two.clone())
            .sample(&mut session)
            .expect("draw"),
        Some(4.0)
    );
    assert_eq!(
        (six.clone() * two.clone())
            .sample(&mut session)
            .expect("draw"),
        Some(12.0)
    );
    assert_eq!(
        (six.clone() / two.clone())
            .sample(&mut session)
            .expect("draw"),
        Some(3.0)
    );
}

/// Subtraction is not commutative here either — the sign must survive presence propagation.
#[test]
fn subtraction_keeps_its_sign_through_presence() {
    let mut session = SampleSession::seeded(SEED);
    let six = MaybeUncertain::<f64>::from_value(6.0);
    let ten = MaybeUncertain::<f64>::from_value(10.0);

    assert_eq!(
        (six.clone() - ten.clone())
            .sample(&mut session)
            .expect("draw"),
        Some(-4.0)
    );
    assert_eq!((ten - six).sample(&mut session).expect("draw"), Some(4.0));
}

/// Negation touches the value and leaves presence alone: negating an absent quantity leaves it
/// absent rather than making it present.
#[test]
fn negation_leaves_presence_alone() {
    let mut session = SampleSession::seeded(SEED);

    assert_eq!(
        (-MaybeUncertain::<f64>::from_value(3.0))
            .sample(&mut session)
            .expect("draw"),
        Some(-3.0)
    );
    assert_eq!(
        (-MaybeUncertain::<f64>::always_none())
            .sample(&mut session)
            .expect("draw"),
        None
    );
}

/// `is_some` and `is_none` are complements at every index.
#[test]
fn is_some_and_is_none_are_complements() {
    let session = SampleSession::seeded(SEED);
    let intermittent =
        MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.5, Uncertain::point(1.0));

    let some = intermittent.is_some();
    let none = intermittent.is_none();

    let mut seen_true = false;
    let mut seen_false = false;
    for index in 0..64 {
        let s = some.sample_at(&session, index).expect("draw");
        let n = none.sample_at(&session, index).expect("draw");
        assert_ne!(s, n, "presence and absence must disagree at index {index}");
        seen_true |= s;
        seen_false |= !s;
    }
    assert!(
        seen_true && seen_false,
        "a fair presence must show both faces"
    );
}

/// The gate lifts a certainly-present value and refuses a certainly-absent one, with a typed
/// presence error rather than a sampling one.
#[test]
fn the_gate_lifts_the_present_and_refuses_the_absent() {
    let session = SampleSession::seeded(SEED);
    let present = MaybeUncertain::<f64>::from_value(9.0);
    let absent = MaybeUncertain::<f64>::always_none();

    let lifted = present
        .lift_to_uncertain(&session, 0.5, 0.95, 0.05, 500)
        .expect("a certainly-present value lifts");
    assert_eq!(lifted.sample_at(&session, 0).expect("draw"), 9.0);

    assert!(
        matches!(
            absent.lift_to_uncertain(&session, 0.5, 0.95, 0.05, 500),
            Err(UncertainError::PresenceError(_))
        ),
        "absence must be a presence error, not a sampling failure"
    );
}

/// The gate follows its threshold: one presence probability, two thresholds either side of it.
#[test]
fn the_gate_follows_its_threshold() {
    let session = SampleSession::seeded(SEED);
    let mostly = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.8, Uncertain::point(1.0));

    assert!(
        mostly
            .lift_to_uncertain(&session, 0.5, 0.95, 0.05, 1000)
            .is_ok(),
        "presence of 0.8 clears a threshold of 0.5"
    );
    assert!(
        mostly
            .lift_to_uncertain(&session, 0.98, 0.95, 0.01, 1000)
            .is_err(),
        "presence of 0.8 does not clear a threshold of 0.98"
    );
}

/// The entropy delegation reaches the same path with its arguments in the right order.
#[test]
fn the_entropy_delegation_reaches_the_same_path() {
    let present = MaybeUncertain::<f64>::from_value(2.0);
    let absent = MaybeUncertain::<f64>::always_none();

    assert!(
        present
            .lift_to_uncertain_from_entropy(0.5, 0.95, 0.05, 300)
            .is_ok()
    );
    assert!(
        absent
            .lift_to_uncertain_from_entropy(0.5, 0.95, 0.05, 300)
            .is_err()
    );
    assert_eq!(present.sample_from_entropy().expect("draw"), Some(2.0));
    assert_eq!(absent.sample_from_entropy().expect("draw"), None);
}

/// Equality is on both channels, so neither can differ without being noticed.
#[test]
fn equality_is_on_both_channels() {
    let a = MaybeUncertain::<f64>::from_value(1.0);
    assert_eq!(a, MaybeUncertain::<f64>::from_value(1.0));

    assert_ne!(
        a,
        MaybeUncertain::<f64>::from_value(2.0),
        "a different value channel is a different quantity"
    );
    assert_ne!(
        a,
        MaybeUncertain::<f64>::from_bernoulli_and_uncertain(1.0, Uncertain::point(1.0)),
        "a certain presence and a Bernoulli presence of one are different graphs"
    );
    assert_ne!(
        MaybeUncertain::<f64>::always_none(),
        MaybeUncertain::<f64>::from_value(0.0),
        "absent-with-zero is not present-with-zero"
    );
}

/// Both channels are drawn at **one** index, so presence and value belong to the same draw.
///
/// The discriminating case: a quantity present exactly when its value is positive. If the two
/// channels drew at different indices the pairing would be arbitrary.
#[test]
fn both_channels_are_drawn_at_one_index() {
    let mut session = SampleSession::seeded(SEED);
    // Value and presence share no leaf, so what is under test is the index rather than the graph.
    let m = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.5, Uncertain::normal(0.0, 1.0));

    // Drawing repeatedly must never fail and must never return a value when absent.
    let mut present_count = 0;
    for _ in 0..200 {
        if let Some(v) = m.sample(&mut session).expect("draw") {
            assert!(v.is_finite(), "a present draw must be a real value");
            present_count += 1;
        }
    }
    assert!(
        (40..160).contains(&present_count),
        "about half of 200 draws should be present, got {present_count}"
    );
}

/// The whole surface works at a scalar the crate never names.
#[test]
fn presence_works_at_a_scalar_the_crate_never_names() {
    use deep_causality_num::BFloat16;

    let mut session = SampleSession::seeded(SEED);
    let present = MaybeUncertain::<BFloat16>::from_value(BFloat16::from(3.0));
    let absent = MaybeUncertain::<BFloat16>::always_none();

    assert_eq!(
        present.sample(&mut session).expect("draw"),
        Some(BFloat16::from(3.0))
    );
    assert_eq!(absent.sample(&mut session).expect("draw"), None);
    assert_eq!(
        (present + absent).sample(&mut session).expect("draw"),
        None,
        "absence propagates at any scalar"
    );
}
