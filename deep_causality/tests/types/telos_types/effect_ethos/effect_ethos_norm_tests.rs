/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::telos_types::effect_ethos::utils_tests;
use deep_causality::{DeonticError, TeloidModal};

#[test]
fn test_add_norm_success() {
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

    assert!(ethos.get_norm(1).is_some());
    assert_eq!(ethos.get_norm(1).unwrap().id(), 1);
}

#[test]
fn test_add_norm_duplicate_id_fails() {
    let result = utils_tests::TestEthos::new()
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
            1,
            "drive",
            &[],
            utils_tests::always_true_predicate,
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
