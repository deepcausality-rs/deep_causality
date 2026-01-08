/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::Uncertain;
use rusty_fork::rusty_fork_test;

rusty_fork_test! {
    #[test]
    fn test_uncertain_bool_bitand() {
        let t = Uncertain::<bool>::point(true);
        let f = Uncertain::<bool>::point(false);

        assert!((t.clone() & t.clone()).sample().unwrap());
        assert!(!(t.clone() & f.clone()).sample().unwrap());
        assert!(!(f.clone() & t.clone()).sample().unwrap());
        assert!(!(f.clone() & f.clone()).sample().unwrap());
    }

    #[test]
    fn test_uncertain_bool_bitor() {
        let t = Uncertain::<bool>::point(true);
        let f = Uncertain::<bool>::point(false);

        assert!((t.clone() | t.clone()).sample().unwrap());
        assert!((t.clone() | f.clone()).sample().unwrap());
        assert!((f.clone() | t.clone()).sample().unwrap());
        assert!(!(f.clone() | f.clone()).sample().unwrap());
    }

    #[test]
    fn test_uncertain_bool_not() {
        let t = Uncertain::<bool>::point(true);
        let f = Uncertain::<bool>::point(false);

        assert!(!(!t).sample().unwrap());
        assert!((!f).sample().unwrap());
    }

    #[test]
    fn test_uncertain_bool_bitxor() {
        let t = Uncertain::<bool>::point(true);
        let f = Uncertain::<bool>::point(false);

        assert!(!(t.clone() ^ t.clone()).sample().unwrap());
        assert!((t.clone() ^ f.clone()).sample().unwrap());
        assert!((f.clone() ^ t.clone()).sample().unwrap());
        assert!(!(f.clone() ^ f.clone()).sample().unwrap());
    }
}
