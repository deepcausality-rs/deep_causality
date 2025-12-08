/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::Uncertain;
use deep_causality::{BaseContext, CausalityError, ProposedAction, UncertainParameter};
use deep_causality_ethos::utils_test::test_utils_effect_ethos;
use deep_causality_ethos::{DeonticError, TeloidModal};

#[test]
fn test_add_deterministic_norm_success() {
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

    assert!(ethos.get_norm(1).is_some());
    assert_eq!(ethos.get_norm(1).unwrap().id(), 1);
}

#[test]
fn test_add_deterministic_norm_duplicate_id_fails() {
    let result = test_utils_effect_ethos::TestEthos::new()
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
            1,
            "drive",
            &[],
            test_utils_effect_ethos::always_true_predicate,
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
fn test_add_uncertain_norm_success() {
    let always_uncertain_predicate: fn(
        &BaseContext,
        &ProposedAction,
    ) -> Result<Uncertain<bool>, CausalityError> =
        |_context, _action| Ok(Uncertain::<bool>::point(true));

    let ethos = test_utils_effect_ethos::TestEthos::new()
        .add_uncertain_norm(
            1,
            "drive",
            &["drive"],
            always_uncertain_predicate,
            UncertainParameter::default(),
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
fn test_add_uncertain_norm_duplicate_id_fails() {
    let always_uncertain_predicate: fn(
        &BaseContext,
        &ProposedAction,
    ) -> Result<Uncertain<bool>, CausalityError> =
        |_context, _action| Ok(Uncertain::<bool>::point(true));

    let result = test_utils_effect_ethos::TestEthos::new()
        .add_uncertain_norm(
            1,
            "drive",
            &[],
            always_uncertain_predicate,
            UncertainParameter::default(),
            TeloidModal::Obligatory,
            1,
            1,
            1,
        )
        .unwrap()
        .add_uncertain_norm(
            1,
            "drive",
            &[],
            always_uncertain_predicate,
            UncertainParameter::default(),
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
