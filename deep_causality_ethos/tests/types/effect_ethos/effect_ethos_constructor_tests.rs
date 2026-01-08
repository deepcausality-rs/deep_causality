/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_ethos::utils_test::test_utils_effect_ethos;
use deep_causality_ethos::*;

#[test]
fn test_new_and_is_verified() {
    let ethos = test_utils_effect_ethos::TestEthos::new();
    assert!(!ethos.is_verified(), "A new ethos should not be verified");
    assert!(ethos.get_norm(1).is_none(), "A new ethos should be empty");
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
    let teloid = Teloid::new_deterministic(
        42,
        "test_norm".to_string(),
        test_utils_effect_ethos::always_true_predicate,
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
    let mut ethos = test_utils_effect_ethos::TestEthos::from(teloid_store, tag_index, teloid_graph);

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
    let action = test_utils_effect_ethos::get_dummy_action("test_action", 0.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["test_tag"];
    let result = ethos.evaluate_action(&action, &context, &tags);

    assert!(result.is_err());
    // The error occurs during conflict resolution when trying to find the node index for ID 42.
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::TeloidNotFound { id: 42 }
    ));
}
