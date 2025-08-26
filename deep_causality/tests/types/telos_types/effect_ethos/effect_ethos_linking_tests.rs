/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::telos_types::effect_ethos::utils_tests;
use deep_causality::{DeonticError, TeloidModal};

#[test]
fn test_linking_success() {
    let ethos_result = utils_tests::TestEthos::new()
        .add_norm(
            1,
            "drive",
            &[],
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
            &[],
            utils_tests::always_true_predicate,
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
    let ethos_result = utils_tests::TestEthos::new()
        .add_norm(
            1,
            "drive",
            &[],
            utils_tests::always_true_predicate,
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
