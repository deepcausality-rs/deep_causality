/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ethos::utils_test::test_utils_effect_ethos;
use deep_causality_ethos::{DeonticError, TeloidModal};

#[test]
fn test_verify_graph_leaves_frozen_on_cycle() {
    // Create an ethos with a cycle
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "a",
            &[],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_deterministic_norm(
            2,
            "b",
            &[],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .link_inheritance(1, 2)
        .unwrap()
        .link_inheritance(2, 1) // Creates a cycle
        .unwrap();

    // Verify initial state
    assert!(!ethos.is_frozen(), "Graph should not be frozen initially");
    assert!(
        !ethos.is_verified(),
        "Graph should not be verified initially"
    );

    // Try to verify the graph - this should detect the cycle
    let result = ethos.verify_graph();

    // The verification should fail with GraphIsCyclic error
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::GraphIsCyclic(_)
    ));

    // BUG CHECK: The graph should NOT be frozen after a failed verification
    assert!(
        !ethos.is_frozen(),
        "Graph should be unfrozen after failed verification"
    );

    // And it's correctly not verified
    assert!(
        !ethos.is_verified(),
        "Graph should not be verified when cycle detected"
    );
}
