/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::telos_types::effect_ethos::utils_tests;
use deep_causality::{DeonticError, TeloidModal};

#[test]
fn test_verify_graph_and_freeze() {
    let mut ethos = utils_tests::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &[],
            utils_tests::always_true_predicate,
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
    let mut ethos = utils_tests::TestEthos::new()
        .add_deterministic_norm(
            1,
            "a",
            &[],
            utils_tests::always_true_predicate,
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
            utils_tests::always_true_predicate,
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
