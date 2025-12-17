/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ethos::utils_test::test_utils_effect_ethos;
use deep_causality_ethos::utils_test::test_utils_effect_ethos::TestEthos;
use deep_causality_ethos::{DeonticError, TeloidModal};

#[test]
fn test_unfreeze() {
    let mut ethos = TestEthos::new()
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
        .unwrap();

    // Verify the graph, which also freezes it and sets is_verified to true.
    ethos.verify_graph().unwrap();
    assert!(ethos.is_frozen());
    assert!(ethos.is_verified());

    // Now, unfreeze it.
    ethos.unfreeze();

    // Check that it's no longer frozen and no longer verified.
    assert!(!ethos.is_frozen());
    assert!(!ethos.is_verified());
}

#[test]
fn test_link_inheritance_fails_if_frozen() {
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
        .unwrap();

    ethos.verify_graph().unwrap(); // This freezes the graph

    let result = ethos.link_inheritance(1, 2);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphIsFrozen(_)));
}

#[test]
fn test_link_defeasance_fails_if_frozen() {
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
        .unwrap();

    ethos.verify_graph().unwrap(); // This freezes the graph

    let result = ethos.link_defeasance(1, 2);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphIsFrozen(_)));
}
