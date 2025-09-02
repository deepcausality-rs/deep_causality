/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::Uncertain;
use rusty_fork::rusty_fork_test;

rusty_fork_test! {
    #[test]
    fn test_uncertain_f64_add() {
        let a = Uncertain::<f64>::point(5.0);
        let b = Uncertain::<f64>::point(3.0);
        let c = a + b;
        assert_eq!(c.sample().unwrap(), 8.0);
    }

    #[test]
    fn test_uncertain_f64_sub() {
        let a = Uncertain::<f64>::point(5.0);
        let b = Uncertain::<f64>::point(3.0);
        let c = a - b;
        assert_eq!(c.sample().unwrap(), 2.0);
    }

    #[test]
    fn test_uncertain_f64_mul() {
        let a = Uncertain::<f64>::point(5.0);
        let b = Uncertain::<f64>::point(3.0);
        let c = a * b;
        assert_eq!(c.sample().unwrap(), 15.0);
    }

    #[test]
    fn test_uncertain_f64_div() {
        let a = Uncertain::<f64>::point(6.0);
        let b = Uncertain::<f64>::point(3.0);
        let c = a / b;
        assert_eq!(c.sample().unwrap(), 2.0);
    }

    #[test]
    fn test_uncertain_f64_neg() {
        let a = Uncertain::<f64>::point(5.0);
        let b = -a;
        assert_eq!(b.sample().unwrap(), -5.0);

        let c = Uncertain::<f64>::point(-10.0);
        let d = -c;
        assert_eq!(d.sample().unwrap(), 10.0);
    }
}
