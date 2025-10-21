/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::MaybeUncertain;
use rusty_fork::rusty_fork_test;

rusty_fork_test! {

    #[test]
    fn test_add_both_some() {
        let a = MaybeUncertain::<f64>::from_value(3.0);
        let b = MaybeUncertain::<f64>::from_value(4.0);
        let res = a + b;
        assert_eq!(res.sample().unwrap(), Some(7.0));
    }

    #[test]
    fn test_add_one_none() {
        let a = MaybeUncertain::<f64>::from_value(3.0);
        let b = MaybeUncertain::<f64>::always_none();
        let res = a + b;
        assert_eq!(res.sample().unwrap(), None);
    }

    #[test]
    fn test_add_both_none() {
        let a = MaybeUncertain::<f64>::always_none();
        let b = MaybeUncertain::<f64>::always_none();
        let res = a + b;
        assert_eq!(res.sample().unwrap(), None);
    }

    #[test]
    fn test_sub_both_some() {
        let a = MaybeUncertain::<f64>::from_value(3.0);
        let b = MaybeUncertain::<f64>::from_value(4.0);
        let res = a - b;
        assert_eq!(res.sample().unwrap(), Some(-1.0));
    }

    #[test]
    fn test_sub_one_none() {
        let a = MaybeUncertain::<f64>::from_value(3.0);
        let b = MaybeUncertain::<f64>::always_none();
        let res = a - b;
        assert_eq!(res.sample().unwrap(), None);
    }

    #[test]
    fn test_sub_both_none() {
        let a = MaybeUncertain::<f64>::always_none();
        let b = MaybeUncertain::<f64>::always_none();
        let res = a - b;
        assert_eq!(res.sample().unwrap(), None);
    }

    #[test]
    fn test_mul_both_some() {
        let a = MaybeUncertain::<f64>::from_value(3.0);
        let b = MaybeUncertain::<f64>::from_value(4.0);
        let res = a * b;
        assert_eq!(res.sample().unwrap(), Some(12.0));
    }

    #[test]
    fn test_mul_one_none() {
        let a = MaybeUncertain::<f64>::from_value(3.0);
        let b = MaybeUncertain::<f64>::always_none();
        let res = a * b;
        assert_eq!(res.sample().unwrap(), None);
    }

    #[test]
    fn test_mul_both_none() {
        let a = MaybeUncertain::<f64>::always_none();
        let b = MaybeUncertain::<f64>::always_none();
        let res = a * b;
        assert_eq!(res.sample().unwrap(), None);
    }

    #[test]
    fn test_div_both_some() {
        let a = MaybeUncertain::<f64>::from_value(12.0);
        let b = MaybeUncertain::<f64>::from_value(4.0);
        let res = a / b;
        assert_eq!(res.sample().unwrap(), Some(3.0));
    }

    #[test]
    fn test_div_one_none() {
        let a = MaybeUncertain::<f64>::from_value(12.0);
        let b = MaybeUncertain::<f64>::always_none();
        let res = a / b;
        assert_eq!(res.sample().unwrap(), None);
    }

    #[test]
    fn test_div_both_none() {
        let a = MaybeUncertain::<f64>::always_none();
        let b = MaybeUncertain::<f64>::always_none();
        let res = a / b;
        assert_eq!(res.sample().unwrap(), None);
    }

    #[test]
    fn test_neg_some() {
        let a = MaybeUncertain::<f64>::from_value(3.0);
        let res = -a;
        assert_eq!(res.sample().unwrap(), Some(-3.0));
    }

    #[test]
    fn test_neg_none() {
        let a = MaybeUncertain::<f64>::always_none();
        let res = -a;
        assert_eq!(res.sample().unwrap(), None);
    }
}
