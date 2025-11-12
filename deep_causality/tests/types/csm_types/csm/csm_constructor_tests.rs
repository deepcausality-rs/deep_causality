/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils_csm;

#[test]
fn test_new() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();

    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    assert_eq!(csm.len(), 1)
}

#[test]
fn test_is_empty() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();

    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    assert!(!csm.is_empty())
}
