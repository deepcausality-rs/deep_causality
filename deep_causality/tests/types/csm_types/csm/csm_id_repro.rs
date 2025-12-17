/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils_csm;
use deep_causality::{CSM, CausalState, PropagatingEffect};

#[test]
fn test_csm_id_mismatch_bug() {
    // 1. Create CSM
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);
    // Initial state ID 99
    let cs = CausalState::new(99, version, data.clone(), causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let csm = CSM::new(&[(&cs, &ca)]);

    // 2. Add state with Internal ID 100, but map Key 200
    let cs2 = CausalState::new(100, 2, data.clone(), causaloid, None);
    let ca2 = test_utils_csm::get_test_action();

    // Using new API without index
    csm.add_single_state((cs2, ca2)).unwrap();

    // 3. Try access by Internal ID 100
    let eval_data = PropagatingEffect::from_value(0.6f64);
    let res = csm.eval_single_state(100, &eval_data);

    // Fix verified: Should succeed because it is now keyed by internal ID 100
    assert!(res.is_ok(), "Fix verified: State 100 found via internal ID");
}
