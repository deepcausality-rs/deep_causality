/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_ethos::utils_test::test_utils_effect_ethos;
use deep_causality_ethos::{DeonticError, DeonticInferable, TeloidModal};

#[test]
fn test_evaluate_action_fails_if_not_verified() {
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    // We freeze the graph, but we do not verify it.
    ethos.freeze();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"];

    let result = ethos.evaluate_action(&action, &context, &tags);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphIsCyclic(_))); // is_verified is false, so it fails this check first
}

#[test]
fn test_evaluate_action_fails_if_not_frozen() {
    let ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    // We do NOT call verify_graph, so the graph is not frozen.

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"];

    let result = ethos.evaluate_action(&action, &context, &tags);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphNotFrozen(_)));
}

#[test]
fn test_evaluate_action_impermissible_wins() {
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_deterministic_norm(
            2,
            "drive",
            &["drive"],
            test_utils_effect_ethos::check_speed_predicate, // This will be active
            TeloidModal::Impermissible,
            2,
            2,
            2,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 60.0); // speed > 50.0
    let context = test_utils_effect_ethos::get_dummy_context();
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
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            100,
            1,
            1,
        )
        .unwrap()
        .add_deterministic_norm(
            2,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Impermissible,
            200,
            1,
            1,
        )
        .unwrap()
        .link_defeasance(2, 1)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Impermissible (newer) wins
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
    assert_eq!(verdict.justification(), &vec![2]);
}

#[test]
fn test_evaluate_action_lex_specialis_wins() {
    // More specific norm (ID 2) should defeat general norm (ID 1)
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            100,
            1,
            1,
        )
        .unwrap()
        .add_deterministic_norm(
            2,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Impermissible,
            100,
            10,
            1,
        )
        .unwrap()
        .link_defeasance(2, 1)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Impermissible (more specific) wins
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
    assert_eq!(verdict.justification(), &vec![2]);
}

#[test]
fn test_evaluate_action_lex_superior_wins() {
    // Higher priority norm (ID 2) should defeat lower priority norm (ID 1)
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            100, // timestamp
            1,   // specificity
            1,   // priority
        )
        .unwrap()
        .add_deterministic_norm(
            2,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Impermissible,
            100, // same timestamp
            1,   // same specificity
            10,  // higher priority
        )
        .unwrap()
        .link_defeasance(2, 1)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Impermissible (higher priority) wins
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
    assert_eq!(verdict.justification(), &vec![2]);
}

#[test]
fn test_evaluate_action_with_inheritance() {
    // General Obligation(1) -> Specific Optional(2)
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_deterministic_norm(
            2,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Optional(10),
            2,
            2,
            2,
        )
        .unwrap()
        .link_inheritance(1, 2)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
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
fn test_evaluate_action_deep_inheritance() {
    // Test a chain of inheritance: 1 -> 2 -> 3
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"], // This is the entry point
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_deterministic_norm(
            2,
            "drive",
            &[], // Not tagged, only reachable via inheritance
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Optional(5),
            2,
            2,
            2,
        )
        .unwrap()
        .add_deterministic_norm(
            3,
            "drive",
            &[], // Not tagged, only reachable via inheritance
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            3,
            3,
            3,
        )
        .unwrap()
        .link_inheritance(1, 2)
        .unwrap()
        .link_inheritance(2, 3)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"]; // Only norm 1 will be in the initial active set

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Obligatory has the highest precedence.
    assert_eq!(verdict.outcome(), TeloidModal::Obligatory);
    // Justification should include all norms in the inheritance chain.
    let mut just = verdict.justification().clone();
    just.sort();
    assert_eq!(just, vec![1, 2, 3]);
}

#[test]
fn test_evaluate_action_inheritance_with_defeasance() {
    // Chain: 1 -> 2. Defeater: 3 defeats 2.
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_deterministic_norm(
            2,
            "drive",
            &[],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Optional(10),
            2,
            1,
            1, // low specificity
        )
        .unwrap()
        .add_deterministic_norm(
            3,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Impermissible,
            3,
            10,
            1, // high specificity
        )
        .unwrap()
        .link_inheritance(1, 2)
        .unwrap()
        .link_defeasance(3, 2) // 3 defeats 2
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"]; // Activates norms 1 and 3

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Impermissible from norm 3 wins.
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
    // Justification should include 1 and 3. Norm 2 was defeated and removed.
    let mut just = verdict.justification().clone();
    just.sort();
    assert_eq!(just, vec![1, 3]);
}

#[test]
fn test_evaluate_action_no_relevant_norms_found() {
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"], // The only tag is "drive"
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("fly", 100.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    // Evaluate with a tag that has no associated norms
    let tags = ["fly"];

    let result = ethos.evaluate_action(&action, &context, &tags);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::NoRelevantNormsFound
    ));
}
