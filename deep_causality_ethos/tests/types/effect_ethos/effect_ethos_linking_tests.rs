/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ethos::utils_test::test_utils_effect_ethos;
use deep_causality_ethos::{DeonticError, TeloidModal};

#[test]
fn test_linking_success() {
    let ethos_result = test_utils_effect_ethos::TestEthos::new()
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
        .unwrap()
        .add_deterministic_norm(
            2,
            "drive",
            &[],
            test_utils_effect_ethos::always_true_predicate,
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
    let ethos_result = test_utils_effect_ethos::TestEthos::new()
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
        .unwrap()
        .link_inheritance(1, 99); // 99 does not exist

    assert!(ethos_result.is_err());
    assert!(matches!(
        ethos_result.unwrap_err(),
        DeonticError::TeloidNotFound { id: 99 }
    ));
}

/// An ethos holding norms 10, 20 and 30, added in that order, so they sit at graph indices
/// 0, 1 and 2.
fn three_norms() -> test_utils_effect_ethos::TestEthos {
    let mut ethos = test_utils_effect_ethos::TestEthos::new();
    for id in [10, 20, 30] {
        ethos = ethos
            .add_deterministic_norm(
                id,
                "drive",
                &[],
                test_utils_effect_ethos::always_true_predicate,
                TeloidModal::Obligatory,
                1,
                1,
                1,
            )
            .unwrap();
    }
    ethos
}

#[test]
fn test_link_defeasance_after_link_inheritance_reports_the_conflict_by_index() {
    // Norms 10 and 20 are the first two added: indices 0 and 1.
    let result = three_norms()
        .link_inheritance(10, 20)
        .unwrap()
        .link_defeasance(10, 20);

    assert_eq!(
        result.err(),
        Some(DeonticError::ConflictingRelation { from: 0, to: 1 })
    );
}

#[test]
fn test_link_inheritance_after_link_defeasance_reports_the_conflict_by_index() {
    // Norms 20 and 30 are the second and third added: indices 1 and 2, not the ids 20 and 30.
    let result = three_norms()
        .link_defeasance(20, 30)
        .unwrap()
        .link_inheritance(20, 30);

    assert_eq!(
        result.err(),
        Some(DeonticError::ConflictingRelation { from: 1, to: 2 })
    );
}

#[test]
fn test_linking_the_same_relation_twice_is_accepted() {
    let result = three_norms()
        .link_inheritance(10, 20)
        .unwrap()
        .link_inheritance(10, 20);

    assert!(result.is_ok());
}

#[test]
fn test_linking_the_opposite_direction_is_accepted() {
    // (10, 20) and (20, 10) are two ordered pairs, so each may carry its own relation.
    let result = three_norms()
        .link_inheritance(10, 20)
        .unwrap()
        .link_defeasance(20, 10);

    assert!(result.is_ok());
}
