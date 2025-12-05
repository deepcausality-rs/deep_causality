/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ethos::utils_test::test_utils_effect_ethos;
use deep_causality_ethos::{DeonticError, TeloidModal};

#[test]
fn test_verify_graph_and_freeze() {
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &[],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    assert!(!ethos.is_verified());
    assert!(!ethos.is_frozen());

    let build_result = ethos.verify_graph();
    assert!(build_result.is_ok());

    assert!(ethos.is_verified());
    assert!(ethos.is_frozen());
}

#[test]
fn test_verify_graph_fails_on_cycle() {
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

    let build_result = ethos.verify_graph();
    assert!(build_result.is_err());
    assert!(matches!(
        build_result.unwrap_err(),
        DeonticError::GraphIsCyclic
    ));
    assert!(!ethos.is_verified());
}
