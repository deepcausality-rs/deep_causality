/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_effect_ethos;
use deep_causality::{DeonticError, TeloidModal};

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
