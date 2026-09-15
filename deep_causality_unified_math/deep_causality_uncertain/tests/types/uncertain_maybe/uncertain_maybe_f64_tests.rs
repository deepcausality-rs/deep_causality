/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{MaybeUncertain, SampleSession, Uncertain, UncertainError};

/// Every test below that observes a draw installs this seed first.
///
/// Without it the draws come from OS entropy, and a test that gates on a sampled decision is a
/// coin flip with a good bias rather than an assertion. Measured before seeding: five tests in
/// this crate failed across ~360 runs, and 41 sampled-decision call sites were exposed. The
/// assertions are unchanged; only the entropy source is.
const SEED: u64 = 0x5EED_2026;

// T004: Test for from_value, from_uncertain, and always_none constructors
#[test]
fn test_from_value_constructor() {
    let mut session = SampleSession::seeded(SEED);
    let mu = MaybeUncertain::<f64>::from_value(42.0);
    assert_eq!(mu.sample(&mut session).unwrap(), Some(42.0));
}

#[test]
fn test_from_uncertain_constructor() {
    let mut session = SampleSession::seeded(SEED);
    let u = Uncertain::<f64>::point(42.0);
    let mu = MaybeUncertain::<f64>::from_uncertain(u);
    assert_eq!(mu.sample(&mut session).unwrap(), Some(42.0));
}

#[test]
fn test_always_none_constructor() {
    let mut session = SampleSession::seeded(SEED);
    let mu = MaybeUncertain::<f64>::always_none();
    assert_eq!(mu.sample(&mut session).unwrap(), None);
}

// T005: Test for from_bernoulli_and_uncertain constructor
#[test]
fn test_from_bernoulli_and_uncertain_constructor() {
    let mut session = SampleSession::seeded(SEED);
    let u = Uncertain::<f64>::point(42.0);
    let mu = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(1.0, u);
    assert_eq!(mu.sample(&mut session).unwrap(), Some(42.0));

    let u2 = Uncertain::<f64>::point(42.0);
    let mu2 = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.0, u2);
    assert_eq!(mu2.sample(&mut session).unwrap(), None);
}

// T006: Test for is_some and is_none methods
#[test]
fn test_is_some() {
    let session = SampleSession::seeded(SEED);
    let mu = MaybeUncertain::<f64>::from_value(42.0);
    let is_some = mu.is_some();
    assert!(is_some.sample_at(&session, 0).unwrap());
}

#[test]
fn test_is_none() {
    let session = SampleSession::seeded(SEED);
    let mu = MaybeUncertain::<f64>::always_none();
    let is_none = mu.is_none();
    assert!(is_none.sample_at(&session, 0).unwrap());
}

// T007: Test for sample method
#[test]
fn test_sample() {
    let mut session = SampleSession::seeded(SEED);
    let mu = MaybeUncertain::<f64>::from_value(123.0);
    assert_eq!(mu.sample(&mut session).unwrap(), Some(123.0));
}
// T008: Test for lift_to_uncertain
#[test]
fn test_lift_to_uncertain_success() {
    let session = SampleSession::seeded(SEED);
    let u = Uncertain::<f64>::point(42.0);
    let mu = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.9, u);
    let result = mu
        .lift_to_uncertain(&session, 0.8, 0.95, 0.05, 100)
        .unwrap();
    assert_eq!(result.sample_at(&session, 0).unwrap(), 42.0);
}

#[test]
fn test_lift_to_uncertain_failure() {
    let session = SampleSession::seeded(SEED);
    let u = Uncertain::<f64>::point(42.0);
    let mu = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.7, u);
    let result = mu.lift_to_uncertain(&session, 0.8, 0.95, 0.05, 100);
    assert!(matches!(result, Err(UncertainError::PresenceError(_))));
}

#[test]
fn test_lift_to_uncertain_always_none() {
    let session = SampleSession::seeded(SEED);
    let mu = MaybeUncertain::<f64>::always_none();
    let result = mu.lift_to_uncertain(&session, 0.1, 0.95, 0.05, 100);
    assert!(matches!(result, Err(UncertainError::PresenceError(_))));
}

#[test]
fn test_lift_to_uncertain_always_some() {
    let session = SampleSession::seeded(SEED);
    let mu = MaybeUncertain::<f64>::from_value(42.0);
    let result = mu
        .lift_to_uncertain(&session, 0.9, 0.95, 0.05, 100)
        .unwrap();
    assert_eq!(result.sample_at(&session, 0).unwrap(), 42.0);
}
