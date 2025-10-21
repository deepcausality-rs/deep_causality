/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{MaybeUncertain, Uncertain, UncertainError};
use rusty_fork::rusty_fork_test;

rusty_fork_test! {
    // T004: Test for from_value, from_uncertain, and always_none constructors
    #[test]
    fn test_from_value_constructor() {
        let mu = MaybeUncertain::<f64>::from_value(42.0);
        assert_eq!(mu.sample().unwrap(), Some(42.0));
    }

    #[test]
    fn test_from_uncertain_constructor() {
        let u = Uncertain::<f64>::point(42.0);
        let mu = MaybeUncertain::<f64>::from_uncertain(u);
        assert_eq!(mu.sample().unwrap(), Some(42.0));
    }

    #[test]
    fn test_always_none_constructor() {
        let mu = MaybeUncertain::<f64>::always_none();
        assert_eq!(mu.sample().unwrap(), None);
    }

    // T005: Test for from_bernoulli_and_uncertain constructor
    #[test]
    fn test_from_bernoulli_and_uncertain_constructor() {
        let u = Uncertain::<f64>::point(42.0);
        let mu = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(1.0, u);
        assert_eq!(mu.sample().unwrap(), Some(42.0));

        let u2 = Uncertain::<f64>::point(42.0);
        let mu2 = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.0, u2);
        assert_eq!(mu2.sample().unwrap(), None);
    }

    // T006: Test for is_some and is_none methods
    #[test]
    fn test_is_some() {
        let mu = MaybeUncertain::<f64>::from_value(42.0);
        let is_some = mu.is_some();
        assert!(is_some.sample().unwrap());
    }

    #[test]
    fn test_is_none() {
        let mu = MaybeUncertain::<f64>::always_none();
        let is_none = mu.is_none();
        assert!(is_none.sample().unwrap());
    }

    // T007: Test for sample method
    #[test]
    fn test_sample() {
        let mu = MaybeUncertain::<f64>::from_value(123.0);
        assert_eq!(mu.sample().unwrap(), Some(123.0));
    }
    // T008: Test for lift_to_uncertain
    #[test]
    fn test_lift_to_uncertain_success() {
        let u = Uncertain::<f64>::point(42.0);
        let mu = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.9, u);
        let result = mu.lift_to_uncertain(0.8, 0.95,0.05,100).unwrap();
        assert_eq!(result.sample().unwrap(), 42.0);
    }

    #[test]
    fn test_lift_to_uncertain_failure() {
        let u = Uncertain::<f64>::point(42.0);
        let mu = MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.7, u);
        let result = mu.lift_to_uncertain(0.8, 0.95,0.05, 100);
        assert!(matches!(result, Err(UncertainError::PresenceError(_))));
    }

    #[test]
    fn test_lift_to_uncertain_always_none() {
        let mu = MaybeUncertain::<f64>::always_none();
        let result = mu.lift_to_uncertain(0.1, 0.95,0.05, 100);
        assert!(matches!(result, Err(UncertainError::PresenceError(_))));
    }

    #[test]
    fn test_lift_to_uncertain_always_some() {
        let mu = MaybeUncertain::<f64>::from_value(42.0);
        let result = mu.lift_to_uncertain(0.9, 0.95,0.05, 100).unwrap();
        assert_eq!(result.sample().unwrap(), 42.0);
    }
}
