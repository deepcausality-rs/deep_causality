/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::utils_test::test_utils;
use crate::*;
use std::sync::{Arc, Mutex};

// Type alias for the complex types to improve readability
pub type BaseEffectEthos = EffectEthos<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

pub fn state_action() -> Result<(), ActionError> {
    Ok(())
}

pub fn get_test_action() -> CausalAction {
    CausalAction::new(state_action, "Test action", 1)
}

pub fn get_test_error_action() -> CausalAction {
    fn err_state_action() -> Result<(), ActionError> {
        Err(ActionError("Error".to_string()))
    }

    CausalAction::new(err_state_action, "Test action", 1)
}

// Causaloid that returns a non-deterministic effect
pub fn get_test_probabilistic_causaloid() -> BaseCausaloid {
    fn causal_fn(_: EffectValue) -> PropagatingEffect {
        PropagatingEffect::from_probabilistic(0.5)
    }
    Causaloid::new(99, causal_fn, "Probabilistic Causaloid")
}

pub fn get_test_error_causaloid() -> BaseCausaloid {
    fn causal_fn(_: EffectValue) -> PropagatingEffect {
        PropagatingEffect::from_error(CausalityError::new("Error".to_string()))
    }
    Causaloid::new(78, causal_fn, "Probabilistic Causaloid")
}

pub fn get_effect_ethos(verified: bool, impermissible: bool) -> BaseEffectEthos {
    let modality = if impermissible {
        TeloidModal::Impermissible
    } else {
        TeloidModal::Obligatory
    };

    let mut ethos = EffectEthos::new()
        .add_deterministic_norm(
            1,
            "Test Norm",
            &["test_tag"],
            |_context, _action| true, // Always active
            modality,
            1,
            1,
            1,
        )
        .unwrap();

    if verified {
        ethos.verify_graph().unwrap();
    }

    ethos
}

pub fn get_test_causaloid(with_context: bool) -> BaseCausaloid {
    fn causal_fn(_effect: EffectValue) -> PropagatingEffect {
        PropagatingEffect::from_deterministic(true)
    }

    if with_context {
        let context = Context::with_capacity(1, "Test Context", 5);
        test_utils::get_test_causaloid_deterministic_with_context::<
            Data<NumericalValue>,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
        >(context)
    } else {
        Causaloid::new(1, causal_fn, "Test Causaloid")
    }
}

pub fn get_test_action_with_tracker() -> CausalAction {
    fn action() -> Result<(), ActionError> {
        let tracker: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
        let mut lock = tracker.lock().unwrap();
        *lock = true;
        Ok(())
    }

    CausalAction::new(action, "Tracked Action", 1)
}
