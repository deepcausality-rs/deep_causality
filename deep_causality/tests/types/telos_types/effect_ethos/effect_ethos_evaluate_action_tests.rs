/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::telos_types::effect_ethos::utils_tests;
use crate::types::telos_types::effect_ethos::utils_tests::TestEthos;
use deep_causality::{DeonticError, DeonticInferable, TeloidModal};

#[test]
fn test_evaluate_action_fails_if_not_verified() {
    let mut ethos = utils_tests::TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            utils_tests::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    // We freeze the graph, but we do not verify it.
    ethos.freeze();

    let action = utils_tests::get_dummy_action("drive", 40.0);
    let context = utils_tests::get_dummy_context();
    let tags = ["drive"];

    let result = ethos.evaluate_action(&action, &context, &tags);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphIsCyclic)); // is_verified is false, so it fails this check first
}

#[test]
fn test_evaluate_action_fails_if_not_frozen() {
    let ethos = utils_tests::TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            utils_tests::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    // We do NOT call verify_graph, so the graph is not frozen.

    let action = utils_tests::get_dummy_action("drive", 40.0);
    let context = utils_tests::get_dummy_context();
    let tags = ["drive"];

    let result = ethos.evaluate_action(&action, &context, &tags);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphNotFrozen));
}

#[test]
fn test_evaluate_action_impermissible_wins() {
    let mut ethos = utils_tests::TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            utils_tests::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_norm(
            2,
            "drive",
            &["drive"],
            utils_tests::check_speed_predicate, // This will be active
            TeloidModal::Impermissible,
            2,
            2,
            2,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = utils_tests::get_dummy_action("drive", 60.0); // speed > 50.0
    let context = utils_tests::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    let mut just = verdict.justification().clone();
    just.sort();
    assert_eq!(just, vec![1, 2]);
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
}

#[test]
fn test_evaluate_action_lex_posterior_wins() {
    // Newer norm (ID 2) should defeat older norm (ID 1)
    let mut ethos = utils_tests::TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            utils_tests::always_true_predicate,
            TeloidModal::Obligatory,
            100,
            1,
            1,
        )
        .unwrap()
        .add_norm(
            2,
            "drive",
            &["drive"],
            utils_tests::always_true_predicate,
            TeloidModal::Impermissible,
            200,
            1,
            1,
        )
        .unwrap()
        .link_defeasance(2, 1)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = utils_tests::get_dummy_action("drive", 40.0);
    let context = utils_tests::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Impermissible (newer) wins
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
    assert_eq!(verdict.justification(), &vec![2]);
}

#[test]
fn test_evaluate_action_lex_specialis_wins() {
    // More specific norm (ID 2) should defeat general norm (ID 1)
    let mut ethos = utils_tests::TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            utils_tests::always_true_predicate,
            TeloidModal::Obligatory,
            100,
            1,
            1,
        )
        .unwrap()
        .add_norm(
            2,
            "drive",
            &["drive"],
            utils_tests::always_true_predicate,
            TeloidModal::Impermissible,
            100,
            10,
            1,
        )
        .unwrap()
        .link_defeasance(2, 1)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = utils_tests::get_dummy_action("drive", 40.0);
    let context = utils_tests::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Impermissible (more specific) wins
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
    assert_eq!(verdict.justification(), &vec![2]);
}

#[test]
fn test_evaluate_action_with_inheritance() {
    // General Obligation(1) -> Specific Optional(2)
    let mut ethos = utils_tests::TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            utils_tests::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_norm(
            2,
            "drive",
            &["drive"],
            utils_tests::always_true_predicate,
            TeloidModal::Optional(10),
            2,
            2,
            2,
        )
        .unwrap()
        .link_inheritance(1, 2)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = utils_tests::get_dummy_action("drive", 40.0);
    let context = utils_tests::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Both are active, Obligatory has higher precedence than Optional
    assert_eq!(verdict.outcome(), TeloidModal::Obligatory);
    // Justification should include both, as 2 inherits from 1.
    let mut just = verdict.justification().clone();
    just.sort();
    assert_eq!(just, vec![1, 2]);
}

#[test]
fn test_evaluate_action_no_relevant_norms_found() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"], // The only tag is "drive"
            utils_tests::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    ethos.verify_graph().unwrap();

    let action = utils_tests::get_dummy_action("fly", 100.0);
    let context = utils_tests::get_dummy_context();
    // Evaluate with a tag that has no associated norms
    let tags = ["fly"];

    let result = ethos.evaluate_action(&action, &context, &tags);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::NoRelevantNormsFound
    ));
}
