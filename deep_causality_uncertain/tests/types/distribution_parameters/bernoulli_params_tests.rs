/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::BernoulliParams;

#[test]
fn test_bernoulli_params_new() {
    let p = 0.5;
    let params = BernoulliParams::new(p);
    assert_eq!(params.p, p);

    let p_zero = 0.0;
    let params_zero = BernoulliParams::new(p_zero);
    assert_eq!(params_zero.p, p_zero);

    let p_one = 1.0;
    let params_one = BernoulliParams::new(p_one);
    assert_eq!(params_one.p, p_one);
}

#[test]
fn test_bernoulli_params_debug_clone_copy() {
    let params = BernoulliParams::new(0.75);

    // Test Debug
    assert_eq!(format!("{:?}", params), "BernoulliParams { p: 0.75 }");

    // Test Clone
    let cloned_params = params;
    assert_eq!(cloned_params.p, params.p);

    // Test Copy (by assignment)
    let copied_params = params;
    assert_eq!(copied_params.p, params.p);
}

#[test]
fn test_bernoulli_params_display() {
    let params = BernoulliParams::new(0.25);
    assert_eq!(format!("{}", params), "BernoulliParams { p: 0.25 }");

    let params_zero = BernoulliParams::new(0.0);
    assert_eq!(format!("{}", params_zero), "BernoulliParams { p: 0.00 }");

    let params_one = BernoulliParams::new(1.0);
    assert_eq!(format!("{}", params_one), "BernoulliParams { p: 1.00 }");
}
