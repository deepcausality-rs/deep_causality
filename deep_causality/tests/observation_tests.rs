/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use deep_causality::prelude::*;
use deep_causality::utils::test_utils::get_test_observation;

#[test]
fn test_effect_observed() {
    let o1 = get_test_observation();
    let target_threshold = 14.0;
    let false_effect = 0.0;
    let true_effect = 1.0;

    assert!(!o1.effect_observed(target_threshold, false_effect));
    assert!(o1.effect_observed(target_threshold, true_effect));
}
