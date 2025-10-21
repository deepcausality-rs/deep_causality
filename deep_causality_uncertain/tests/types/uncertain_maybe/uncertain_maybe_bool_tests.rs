/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{MaybeUncertain, Uncertain, UncertainError};

#[test]
fn test_from_uncertain() {
    let uncertain_true = Uncertain::<bool>::point(true);
    let maybe_uncertain = MaybeUncertain::<bool>::from_uncertain(uncertain_true.clone());
    assert!(
        maybe_uncertain
            .is_some()
            .to_bool(0.5, 0.95, 0.05, 1000)
            .unwrap()
    );

    let uncertain_false = Uncertain::<bool>::point(false);
    let maybe_uncertain = MaybeUncertain::<bool>::from_uncertain(uncertain_false.clone());
    assert!(
        maybe_uncertain
            .is_some()
            .to_bool(0.5, 0.95, 0.05, 1000)
            .unwrap()
    );
}

#[test]
fn test_from_value() {
    let maybe_uncertain_true = MaybeUncertain::<bool>::from_value(true);
    assert!(
        maybe_uncertain_true
            .is_some()
            .to_bool(0.5, 0.95, 0.05, 1000)
            .unwrap()
    );

    let maybe_uncertain_false = MaybeUncertain::<bool>::from_value(false);
    assert!(
        maybe_uncertain_false
            .is_some()
            .to_bool(0.5, 0.95, 0.05, 1000)
            .unwrap()
    );
}

#[test]
fn test_always_none() {
    let maybe_uncertain = MaybeUncertain::<bool>::always_none();
    assert!(
        !maybe_uncertain
            .is_some()
            .to_bool(0.5, 0.95, 0.05, 1000)
            .unwrap()
    );
    // The value field is a dummy in this case, so we don't assert its specific value
}

#[test]
fn test_from_bernoulli_and_uncertain() {
    let present_value_dist = Uncertain::<bool>::point(true);
    let maybe_uncertain =
        MaybeUncertain::<bool>::from_bernoulli_and_uncertain(0.8, present_value_dist.clone());
    // is_present should be a Bernoulli distribution with p=0.8
    // We can't directly assert the distribution type, but we can sample it.
    // For 100% coverage, we need to ensure the bernoulli branch is taken.
    // This is hard to test deterministically without mocking, so we'll rely on sampling.
    let mut num_some = 0;
    for _ in 0..1000 {
        if maybe_uncertain.is_some().sample().unwrap() {
            num_some += 1;
        }
    }
    assert!(num_some > 700 && num_some < 900); // Roughly 80% should be true

    let present_value_dist_false = Uncertain::<bool>::point(false);
    let maybe_uncertain_false =
        MaybeUncertain::<bool>::from_bernoulli_and_uncertain(0.2, present_value_dist_false.clone());
    let mut num_some_false = 0;
    for _ in 0..1000 {
        if maybe_uncertain_false.is_some().sample().unwrap() {
            num_some_false += 1;
        }
    }
    assert!(num_some_false > 100 && num_some_false < 300); // Roughly 20% should be true
}

#[test]
fn test_sample() -> Result<(), UncertainError> {
    // Test case 1: is_present is true, value is true
    let maybe_uncertain_true = MaybeUncertain::<bool>::from_value(true);
    let mut num_true_samples = 0;
    let mut num_none_samples = 0;
    for _ in 0..1000 {
        match maybe_uncertain_true.sample()? {
            Some(true) => num_true_samples += 1,
            None => num_none_samples += 1,
            _ => {}
        }
    }
    assert_eq!(num_true_samples, 1000); // Always true
    assert_eq!(num_none_samples, 0);

    // Test case 2: is_present is true, value is false
    let maybe_uncertain_false = MaybeUncertain::<bool>::from_value(false);
    num_true_samples = 0;
    num_none_samples = 0;
    for _ in 0..1000 {
        match maybe_uncertain_false.sample()? {
            Some(false) => num_true_samples += 1,
            None => num_none_samples += 1,
            _ => {}
        }
    }
    assert_eq!(num_true_samples, 1000); // Always false
    assert_eq!(num_none_samples, 0);

    // Test case 3: is_present is false (always_none)
    let maybe_uncertain_none = MaybeUncertain::<bool>::always_none();
    num_true_samples = 0;
    num_none_samples = 0;
    for _ in 0..1000 {
        match maybe_uncertain_none.sample()? {
            Some(_) => num_true_samples += 1,
            None => num_none_samples += 1,
        }
    }
    assert_eq!(num_true_samples, 0);
    assert_eq!(num_none_samples, 1000);

    // Test case 4: is_present is bernoulli, value is true
    let present_value_dist = Uncertain::<bool>::point(true);
    let maybe_uncertain_bernoulli =
        MaybeUncertain::<bool>::from_bernoulli_and_uncertain(0.5, present_value_dist);
    num_true_samples = 0;
    num_none_samples = 0;
    for _ in 0..1000 {
        match maybe_uncertain_bernoulli.sample()? {
            Some(true) => num_true_samples += 1,
            None => num_none_samples += 1,
            _ => {}
        }
    }
    assert!(num_true_samples > 400 && num_true_samples < 600); // Roughly 50% true
    assert!(num_none_samples > 400 && num_none_samples < 600); // Roughly 50% none

    Ok(())
}

#[test]
fn test_is_some() {
    let uncertain_true = Uncertain::<bool>::point(true);
    let maybe_uncertain = MaybeUncertain::<bool>::from_uncertain(uncertain_true.clone());
    assert_eq!(maybe_uncertain.is_some(), uncertain_true);

    let maybe_uncertain_none = MaybeUncertain::<bool>::always_none();
    assert!(
        !maybe_uncertain_none
            .is_some()
            .to_bool(0.5, 0.95, 0.05, 1000)
            .unwrap()
    );
}

#[test]
fn test_is_none() {
    let uncertain_true = Uncertain::<bool>::point(true);
    let maybe_uncertain = MaybeUncertain::<bool>::from_uncertain(uncertain_true.clone());
    assert!(
        !maybe_uncertain
            .is_none()
            .to_bool(0.5, 0.95, 0.05, 1000)
            .unwrap()
    );

    let maybe_uncertain_none = MaybeUncertain::<bool>::always_none();
    assert!(
        maybe_uncertain_none
            .is_none()
            .to_bool(0.5, 0.95, 0.05, 1000)
            .unwrap()
    );
}

#[test]
fn test_lift_to_uncertain() -> Result<(), UncertainError> {
    // Test case 1: is_present is true, should lift successfully
    let uncertain_true_value = Uncertain::<bool>::point(true);
    let maybe_uncertain_present = MaybeUncertain::<bool>::from_value(true);
    let lifted_uncertain = maybe_uncertain_present.lift_to_uncertain(0.5, 0.95, 0.05, 1000)?;
    assert_eq!(lifted_uncertain, uncertain_true_value);

    // Test case 2: is_present is false, should return PresenceError
    let maybe_uncertain_absent = MaybeUncertain::<bool>::always_none();
    let result = maybe_uncertain_absent.lift_to_uncertain(0.5, 0.95, 0.05, 1000);
    assert!(matches!(result, Err(UncertainError::PresenceError(_))));

    // Test case 3: is_present is bernoulli, and passes threshold
    let present_value_dist = Uncertain::<bool>::point(true);
    let maybe_uncertain_bernoulli_pass =
        MaybeUncertain::<bool>::from_bernoulli_and_uncertain(0.9, present_value_dist.clone());
    let lifted_uncertain_bernoulli_pass =
        maybe_uncertain_bernoulli_pass.lift_to_uncertain(0.5, 0.95, 0.05, 1000)?;
    assert_eq!(lifted_uncertain_bernoulli_pass, present_value_dist);

    // Test case 4: is_present is bernoulli, and fails threshold
    let maybe_uncertain_bernoulli_fail =
        MaybeUncertain::<bool>::from_bernoulli_and_uncertain(0.1, present_value_dist);
    let result_fail = maybe_uncertain_bernoulli_fail.lift_to_uncertain(0.5, 0.95, 0.05, 1000);
    assert!(matches!(result_fail, Err(UncertainError::PresenceError(_))));

    Ok(())
}
