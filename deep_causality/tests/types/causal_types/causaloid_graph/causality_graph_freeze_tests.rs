/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::{CausableGraph, CausaloidGraph};

#[test]
fn test_freeze_unfreeze() {
    let mut g = CausaloidGraph::new(0);

    // Add root causaloid
    let root_causaloid = test_utils::get_test_causaloid_deterministic(0);
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    // Add causaloid A
    let causaloid = test_utils::get_test_causaloid_deterministic(1);
    let idx_a = g.add_causaloid(causaloid).expect("Failed to add causaloid");

    g.freeze();

    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    g.unfreeze();

    let causaloid = test_utils::get_test_causaloid_deterministic(2);
    let idx_b = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let res = g.add_edge(root_index, idx_b);
    assert!(res.is_ok());

    g.freeze();

    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);
}
