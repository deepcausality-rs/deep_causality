/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::Uncertain;
use rusty_fork::rusty_fork_test;

rusty_fork_test! {

    #[test]
    fn test_uncertain_f64_default() {
        let a = Uncertain::<f64>::point(0.0);
        let b = Uncertain::<f64>::default();
        assert_eq!(a,b);
    }

        #[test]
    fn test_uncertain_bool_default() {
        let a = Uncertain::<bool>::point(true);
        let b = Uncertain::<bool>::default();
        assert_eq!(a,b);
    }
}
