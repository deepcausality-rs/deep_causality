/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::Uncertain;
use rusty_fork::rusty_fork_test;

rusty_fork_test! {
    #[test]
    fn test_uncertain_f64_greater_than() {
        let u = Uncertain::<f64>::point(10.0);
        assert!(u.greater_than(5.0).sample().unwrap());
        assert!(!u.greater_than(10.0).sample().unwrap());
        assert!(!u.greater_than(15.0).sample().unwrap());
    }

    #[test]
    fn test_uncertain_f64_less_than() {
        let u = Uncertain::<f64>::point(10.0);
        assert!(u.less_than(15.0).sample().unwrap());
        assert!(!u.less_than(10.0).sample().unwrap());
        assert!(!u.less_than(5.0).sample().unwrap());
    }

    #[test]
    fn test_uncertain_f64_equals() {
        let u = Uncertain::<f64>::point(10.0);
        // Due to f64::EPSILON in ComparisonOperator::EqualTo, direct equality might be tricky.
        // Let's test with values that are exactly equal.
        assert!(u.equals(10.0).sample().unwrap());
        assert!(!u.equals(10.1).sample().unwrap());
    }

    #[test]
    fn test_uncertain_f64_gt_uncertain() {
        let u1 = Uncertain::<f64>::point(10.0);
        let u2 = Uncertain::<f64>::point(5.0);
        let u3 = Uncertain::<f64>::point(10.0);

        assert!(u1.gt_uncertain(&u2).sample().unwrap());
        assert!(!u1.gt_uncertain(&u3).sample().unwrap());
        assert!(!u2.gt_uncertain(&u1).sample().unwrap());
    }

    #[test]
    fn test_uncertain_f64_lt_uncertain() {
        let u1 = Uncertain::<f64>::point(5.0);
        let u2 = Uncertain::<f64>::point(10.0);
        let u3 = Uncertain::<f64>::point(5.0);

        assert!(u1.lt_uncertain(&u2).sample().unwrap());
        assert!(!u1.lt_uncertain(&u3).sample().unwrap());
        assert!(!u2.lt_uncertain(&u1).sample().unwrap());
    }

    #[test]
    fn test_uncertain_f64_eq_uncertain() {
        let u1 = Uncertain::<f64>::point(10.0);
        let u2 = Uncertain::<f64>::point(10.0);
        let u3 = Uncertain::<f64>::point(5.0);

        assert!(u1.eq_uncertain(&u2).sample().unwrap());
        assert!(!u1.eq_uncertain(&u3).sample().unwrap());
    }

    #[test]
    fn test_uncertain_f64_approx_eq() {
        let u = Uncertain::<f64>::point(10.0);
        let tolerance = 0.1;

        assert!(u.approx_eq(10.05, tolerance).sample().unwrap()); // Within tolerance
        assert!(u.approx_eq(9.95, tolerance).sample().unwrap());  // Within tolerance
        assert!(!u.approx_eq(10.2, tolerance).sample().unwrap()); // Outside tolerance
        assert!(!u.approx_eq(9.8, tolerance).sample().unwrap());  // Outside tolerance
    }

    #[test]
    fn test_uncertain_f64_within_range() {
        let u = Uncertain::<f64>::point(10.0);

        assert!(u.within_range(5.0, 15.0).sample().unwrap());
        assert!(u.within_range(10.0, 10.0).sample().unwrap());
        assert!(!u.within_range(1.0, 5.0).sample().unwrap());
        assert!(!u.within_range(15.0, 20.0).sample().unwrap());
    }
}
