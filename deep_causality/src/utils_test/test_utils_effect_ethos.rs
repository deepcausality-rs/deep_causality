/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ActionParameterValue, BaseContext, BaseSymbol, Data, EffectEthos, EuclideanSpace,
    EuclideanSpacetime, EuclideanTime, FloatType, NumericalValue, ProposedAction,
};
use std::collections::HashMap;

// HELPER FUNCTIONS
// Type alias for the standard EffectEthos used in tests
pub type TestEthos = EffectEthos<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

// Predicate that always returns true
pub fn always_true_predicate(_context: &BaseContext, _action: &ProposedAction) -> bool {
    true
}

// Predicate that always returns false
pub fn always_false_predicate(_context: &BaseContext, _action: &ProposedAction) -> bool {
    false
}

// Predicate that checks if a "speed" parameter is greater than 50.0
pub fn check_speed_predicate(_context: &BaseContext, action: &ProposedAction) -> bool {
    if let Some(ActionParameterValue::Number(speed)) = action.parameters().get("speed") {
        *speed > 50.0
    } else {
        false
    }
}

// Creates a dummy context for testing
pub fn get_dummy_context() -> BaseContext {
    BaseContext::with_capacity(0, "dummy_context", 10)
}

// Creates a dummy action for testing predicates
pub fn get_dummy_action(action_name: &str, speed: f64) -> ProposedAction {
    let mut params = HashMap::new();
    params.insert("speed".to_string(), ActionParameterValue::Number(speed));
    ProposedAction::new(0, action_name.to_string(), params)
}
