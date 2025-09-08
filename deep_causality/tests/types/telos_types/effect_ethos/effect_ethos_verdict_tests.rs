/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils_effect_ethos;
use deep_causality::{DeonticError, DeonticExplainable, DeonticInferable, TeloidModal, Verdict};

#[test]
fn test_inconclusive_verdict_no_active_norms() {
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_false_predicate,
            TeloidModal::Impermissible,
            1,
            1,
            1,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
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
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::check_speed_predicate,
            TeloidModal::Impermissible,
            1,
            10,
            1,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 60.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();
    let explanation = ethos.explain_verdict(&verdict).unwrap();

    assert!(explanation.contains("The final verdict is Impermissible."));
    assert!(explanation.contains("Norm 1: 'drive' (Impermissible, Specificity: 10, Timestamp: 1"));
    assert!(explanation.contains("highest precedence"));
}

#[test]
fn test_explain_verdict_obligatory() {
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
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();
    let explanation = ethos.explain_verdict(&verdict).unwrap();

    assert!(explanation.contains("The final verdict is Obligatory."));
    assert!(explanation.contains("Norm 1: 'drive' (Obligatory, Specificity: 1, Timestamp: 1"));
    assert!(explanation.contains("no impermissible norms were found"));
}

#[test]
fn test_explain_verdict_optional() {
    let mut ethos = test_utils_effect_ethos::TestEthos::new()
        .add_deterministic_norm(
            1,
            "drive",
            &["drive"],
            test_utils_effect_ethos::always_true_predicate,
            TeloidModal::Optional(42),
            1,
            1,
            1,
        )
        .unwrap();
    ethos.verify_graph().unwrap();

    let action = test_utils_effect_ethos::get_dummy_action("drive", 40.0);
    let context = test_utils_effect_ethos::get_dummy_context();
    let tags = ["drive"];

    let verdict = ethos.evaluate_action(&action, &context, &tags).unwrap();
    let explanation = ethos.explain_verdict(&verdict).unwrap();

    assert!(explanation.contains("The final verdict is Optional(42)."));
    assert!(explanation.contains("Norm 1: 'drive' (Optional(42), Specificity: 1, Timestamp: 1"));
    assert!(explanation.contains("only optional norms were active"));
}

#[test]
fn test_explain_verdict_teloid_not_found() {
    let ethos = test_utils_effect_ethos::TestEthos::new(); // Empty ethos
    let bad_verdict = Verdict::new(TeloidModal::Obligatory, vec![999]); // Contains non-existent ID

    let result = ethos.explain_verdict(&bad_verdict);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        DeonticError::TeloidNotFound { id: 999 }
    ));
}
