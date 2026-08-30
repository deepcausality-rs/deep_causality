/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `MaybeUncertain<Float106>` — the precision-carrying mirror of the `f64` surface:
//! the four constructors, `sample`, `is_some` / `is_none`, and the SPRT-gated `lift_to_uncertain`
//! (success, presence-failure, always-none, always-some).

use deep_causality_num::Float106;
use deep_causality_uncertain::{MaybeUncertain, Uncertain, UncertainError};
use rusty_fork::rusty_fork_test;

rusty_fork_test! {
    #[test]
    fn test_from_value_constructor() {
        let mu = MaybeUncertain::<Float106>::from_value(Float106::from_f64(42.0));
        assert_eq!(mu.sample().unwrap(), Some(Float106::from_f64(42.0)));
    }

    #[test]
    fn test_from_uncertain_constructor() {
        let u = Uncertain::<Float106>::point(Float106::from_f64(42.0));
        let mu = MaybeUncertain::<Float106>::from_uncertain(u);
        assert_eq!(mu.sample().unwrap(), Some(Float106::from_f64(42.0)));
    }

    #[test]
    fn test_always_none_constructor() {
        let mu = MaybeUncertain::<Float106>::always_none();
        assert_eq!(mu.sample().unwrap(), None);
    }

    #[test]
    fn test_from_bernoulli_and_uncertain_constructor() {
        let u = Uncertain::<Float106>::point(Float106::from_f64(42.0));
        let mu = MaybeUncertain::<Float106>::from_bernoulli_and_uncertain(1.0, u);
        assert_eq!(mu.sample().unwrap(), Some(Float106::from_f64(42.0)));

        let u2 = Uncertain::<Float106>::point(Float106::from_f64(42.0));
        let mu2 = MaybeUncertain::<Float106>::from_bernoulli_and_uncertain(0.0, u2);
        assert_eq!(mu2.sample().unwrap(), None);
    }

    #[test]
    fn test_is_some() {
        let mu = MaybeUncertain::<Float106>::from_value(Float106::from_f64(42.0));
        assert!(mu.is_some().sample().unwrap());
    }

    #[test]
    fn test_is_none() {
        let mu = MaybeUncertain::<Float106>::always_none();
        assert!(mu.is_none().sample().unwrap());
    }

    #[test]
    fn test_sample() {
        let mu = MaybeUncertain::<Float106>::from_value(Float106::from_f64(123.0));
        assert_eq!(mu.sample().unwrap(), Some(Float106::from_f64(123.0)));
    }

    #[test]
    fn test_lift_to_uncertain_success() {
        let u = Uncertain::<Float106>::point(Float106::from_f64(42.0));
        let mu = MaybeUncertain::<Float106>::from_bernoulli_and_uncertain(0.9, u);
        let result = mu.lift_to_uncertain(0.8, 0.95, 0.05, 100).unwrap();
        assert_eq!(result.sample().unwrap(), Float106::from_f64(42.0));
    }

    #[test]
    fn test_lift_to_uncertain_failure() {
        let u = Uncertain::<Float106>::point(Float106::from_f64(42.0));
        let mu = MaybeUncertain::<Float106>::from_bernoulli_and_uncertain(0.7, u);
        let result = mu.lift_to_uncertain(0.8, 0.95, 0.05, 100);
        assert!(matches!(result, Err(UncertainError::PresenceError(_))));
    }

    #[test]
    fn test_lift_to_uncertain_always_none() {
        let mu = MaybeUncertain::<Float106>::always_none();
        let result = mu.lift_to_uncertain(0.1, 0.95, 0.05, 100);
        assert!(matches!(result, Err(UncertainError::PresenceError(_))));
    }

    #[test]
    fn test_lift_to_uncertain_always_some() {
        let mu = MaybeUncertain::<Float106>::from_value(Float106::from_f64(42.0));
        let result = mu.lift_to_uncertain(0.9, 0.95, 0.05, 100).unwrap();
        assert_eq!(result.sample().unwrap(), Float106::from_f64(42.0));
    }
}
