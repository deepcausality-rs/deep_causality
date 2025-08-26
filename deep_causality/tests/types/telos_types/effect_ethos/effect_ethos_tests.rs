/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;
use std::collections::HashMap;

// HELPER FUNCTIONS
// Type alias for the standard EffectEthos used in tests
type TestEthos = EffectEthos<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

// Predicate that always returns true
fn always_true_predicate(_context: &BaseContext, _action: &ProposedAction) -> bool {
    true
}

// Predicate that always returns false
fn always_false_predicate(_context: &BaseContext, _action: &ProposedAction) -> bool {
    false
}

// Predicate that checks if a "speed" parameter is greater than 50.0
fn check_speed_predicate(_context: &BaseContext, action: &ProposedAction) -> bool {
    if let Some(ActionParameterValue::Number(speed)) = action.parameters().get("speed") {
        *speed > 50.0
    } else {
        false
    }
}

// Creates a dummy context for testing
fn get_dummy_context() -> BaseContext {
    BaseContext::with_capacity(0, "dummy_context", 10)
}

// Creates a dummy action for testing predicates
fn get_dummy_action(action_name: &str, speed: f64) -> ProposedAction {
    let mut params = HashMap::new();
    params.insert("speed".to_string(), ActionParameterValue::Number(speed));
    ProposedAction::new(0, action_name.to_string(), params)
}

// TESTS START HERE

#[test]
fn test_new_and_is_verified() {
    let ethos = TestEthos::new();
    assert!(!ethos.is_verified(), "A new ethos should not be verified");
    assert!(ethos.get_norm(1).is_none(), "A new ethos should be empty");
}

#[test]
fn test_add_norm_success() {
    let ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    assert!(ethos.get_norm(1).is_some());
    assert_eq!(ethos.get_norm(1).unwrap().id(), 1);
}

#[test]
fn test_add_norm_duplicate_id_fails() {
    let result = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &[],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_norm(
            1,
            "drive",
            &[],
            always_true_predicate,
            TeloidModal::Impermissible,
            2,
            2,
            2,
        );

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::FailedToAddTeloid
    ));
}

#[test]
fn test_evaluate_action_no_relevant_norms_found() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"], // The only tag is "drive"
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    ethos.verify_graph().unwrap();

    let action = get_dummy_action("fly", 100.0);
    let context = get_dummy_context();
    // Evaluate with a tag that has no associated norms
    let tags = ["fly"];

    let result = ethos.evaluate_action(&action, &context, &tags);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::NoRelevantNormsFound
    ));
}

#[test]
fn test_from_constructor() {
    // This test uses the `from` constructor to build an EffectEthos from
    // its constituent parts, simulating a scenario like deserialization.

    // 1. Manually construct the components.
    // We create a store and index with data, but the graph is empty.
    // This tests the constructor's behavior and its assumption that the
    // provided components are consistent.
    let mut teloid_store = TeloidStore::new();
    let teloid = Teloid::new(
        42,
        "test_norm".to_string(),
        always_true_predicate,
        TeloidModal::Obligatory,
        1,
        1,
        1,
        vec!["test_tag"],
        None,
    );
    teloid_store.insert(teloid);

    let mut tag_index = TagIndex::new();
    tag_index.add("test_tag", 42);

    // The graph is empty, making it inconsistent with the store and index.
    let teloid_graph = TeloidGraph::new();

    // 2. Create an ethos from the components.
    let mut ethos = TestEthos::from(teloid_store, tag_index, teloid_graph);

    // 3. Assert the initial state is as expected.
    assert!(ethos.get_norm(42).is_some(), "Norm should be in the store");
    assert!(
        !ethos.is_verified(),
        "Ethos should not be verified initially"
    );

    // 4. Verification should succeed as the empty graph is valid.
    ethos.verify_graph().unwrap();
    assert!(ethos.is_verified());

    // 5. Now, test the consequence of the inconsistent graph.
    // Evaluation should fail because the internal state was built from the
    // empty graph and thus doesn't contain a node for ID '42'.
    let action = get_dummy_action("test_action", 0.0);
    let context = get_dummy_context();
    let tags = ["test_tag"];
    let result = ethos.evaluate_action(&action, &context, &tags);

    assert!(result.is_err());
    // The error occurs during conflict resolution when trying to find the node index for ID 42.
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::TeloidNotFound { id: 42 }
    ));
}

#[test]
fn test_unfreeze() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "a",
            &[],
            always_true_predicate,
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
fn test_linking_success() {
    let ethos_result = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &[],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_norm(
            2,
            "drive",
            &[],
            always_true_predicate,
            TeloidModal::Impermissible,
            2,
            2,
            2,
        )
        .unwrap()
        .link_inheritance(1, 2);

    assert!(ethos_result.is_ok());
}

#[test]
fn test_linking_fails_on_non_existent_id() {
    let ethos_result = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &[],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .link_inheritance(1, 99); // 99 does not exist

    assert!(ethos_result.is_err());
    assert!(matches!(
        ethos_result.unwrap_err(),
        DeonticError::TeloidNotFound { id: 99 }
    ));
}

#[test]
fn test_verify_graph_and_freeze() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &[],
            always_true_predicate,
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
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "a",
            &[],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_norm(
            2,
            "b",
            &[],
            always_true_predicate,
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

#[test]
fn test_evaluate_action_fails_if_not_verified() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    // We freeze the graph, but we do not verify it.
    ethos.freeze();

    let action = get_dummy_action("drive", 40.0);
    let context = get_dummy_context();
    let tags = ["drive"];

    let result = ethos.evaluate_action(&action, &context, &tags);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphIsCyclic)); // is_verified is false, so it fails this check first
}

#[test]
fn test_evaluate_action_fails_if_not_frozen() {
    let ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    // We do NOT call verify_graph, so the graph is not frozen.

    let action = get_dummy_action("drive", 40.0);
    let context = get_dummy_context();
    let tags = ["drive"];

    let result = ethos.evaluate_action(&action, &context, &tags);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphNotFrozen));
}

#[test]
fn test_evaluate_action_impermissible_wins() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_true_predicate,
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
            check_speed_predicate, // This will be active
            TeloidModal::Impermissible,
            2,
            2,
            2,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = get_dummy_action("drive", 60.0); // speed > 50.0
    let context = get_dummy_context();
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
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_true_predicate,
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
            always_true_predicate,
            TeloidModal::Impermissible,
            200,
            1,
            1,
        )
        .unwrap()
        .link_defeasance(2, 1)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = get_dummy_action("drive", 40.0);
    let context = get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Impermissible (newer) wins
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
    assert_eq!(verdict.justification(), &vec![2]);
}

#[test]
fn test_evaluate_action_lex_specialis_wins() {
    // More specific norm (ID 2) should defeat general norm (ID 1)
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_true_predicate,
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
            always_true_predicate,
            TeloidModal::Impermissible,
            100,
            10,
            1,
        )
        .unwrap()
        .link_defeasance(2, 1)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = get_dummy_action("drive", 40.0);
    let context = get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();

    // Impermissible (more specific) wins
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
    assert_eq!(verdict.justification(), &vec![2]);
}

#[test]
fn test_evaluate_action_with_inheritance() {
    // General Obligation(1) -> Specific Optional(2)
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_true_predicate,
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
            always_true_predicate,
            TeloidModal::Optional(10),
            2,
            2,
            2,
        )
        .unwrap()
        .link_inheritance(1, 2)
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = get_dummy_action("drive", 40.0);
    let context = get_dummy_context();
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
fn test_inconclusive_verdict_no_active_norms() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_false_predicate,
            TeloidModal::Impermissible,
            1,
            1,
            1,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = get_dummy_action("drive", 40.0);
    let context = get_dummy_context();
    let tags = ["drive"];

    let result = ethos.evaluate_action(&action, &context, &tags);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::InconclusiveVerdict
    ));
}

#[test]
fn test_explain_verdict_impermissible() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            check_speed_predicate,
            TeloidModal::Impermissible,
            1,
            10,
            1,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = get_dummy_action("drive", 60.0);
    let context = get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();
    let explanation = ethos.explain_verdict(&verdict).unwrap();

    assert!(explanation.contains("The final verdict is Impermissible."));
    assert!(explanation.contains("Norm 1: 'drive' (Impermissible, Specificity: 10, Timestamp: 1"));
    assert!(explanation.contains("highest precedence"));
}

#[test]
fn test_explain_verdict_obligatory() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = get_dummy_action("drive", 40.0);
    let context = get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();
    let explanation = ethos.explain_verdict(&verdict).unwrap();

    assert!(explanation.contains("The final verdict is Obligatory."));
    assert!(explanation.contains("Norm 1: 'drive' (Obligatory, Specificity: 1, Timestamp: 1"));
    assert!(explanation.contains("no impermissible norms were found"));
}

#[test]
fn test_explain_verdict_optional() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "drive",
            &["drive"],
            always_true_predicate,
            TeloidModal::Optional(42),
            1,
            1,
            1,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = get_dummy_action("drive", 40.0);
    let context = get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();
    let explanation = ethos.explain_verdict(&verdict).unwrap();

    assert!(explanation.contains("The final verdict is Optional(42)."));
    assert!(explanation.contains("Norm 1: 'drive' (Optional(42), Specificity: 1, Timestamp: 1"));
    assert!(explanation.contains("only optional norms were active"));
}

#[test]
fn test_explain_verdict_teloid_not_found() {
    let ethos = TestEthos::new(); // Empty ethos
    let bad_verdict = Verdict::new(TeloidModal::Obligatory, vec![999]); // Contains non-existent ID

    let result = ethos.explain_verdict(&bad_verdict);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::TeloidNotFound { id: 999 }
    ));
}

#[test]
fn test_link_inheritance_fails_if_frozen() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "a",
            &[],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_norm(
            2,
            "b",
            &[],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    ethos.verify_graph().unwrap(); // This freezes the graph

    let result = ethos.link_inheritance(1, 2);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphIsFrozen));
}

#[test]
fn test_link_defeasance_fails_if_frozen() {
    let mut ethos = TestEthos::new()
        .add_norm(
            1,
            "a",
            &[],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_norm(
            2,
            "b",
            &[],
            always_true_predicate,
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap();

    ethos.verify_graph().unwrap(); // This freezes the graph

    let result = ethos.link_defeasance(1, 2);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DeonticError::GraphIsFrozen));
}
