/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{MaybeUncertain, Uncertain, UncertainError};
use rusty_fork::rusty_fork_test;

rusty_fork_test! {

    // T008: Test for lift_to_uncertain
    #[test]
    fn test_lift_to_uncertain_success() {
        let u = Uncertain::<f64>::point(42.0);
        let mu = MaybeUncertain::from_bernoulli_and_uncertain(0.9, u);
        let result = mu.lift_to_uncertain(0.8, 0.95,0.05,100).unwrap();
        assert_eq!(result.sample().unwrap(), 42.0);
    }

    #[test]
    fn test_lift_to_uncertain_failure() {
        let u = Uncertain::<f64>::point(42.0);
        let mu = MaybeUncertain::from_bernoulli_and_uncertain(0.7, u);
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
