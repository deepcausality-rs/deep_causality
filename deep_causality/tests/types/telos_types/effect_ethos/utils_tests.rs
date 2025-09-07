/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    ActionParameterValue, BaseContext, BaseSymbol, CausalityError, Data, EffectEthos,
    EuclideanSpace, EuclideanSpacetime, EuclideanTime, FloatType, NumericalValue, ProposedAction,
};
use deep_causality_uncertain::Uncertain;
use std::collections::HashMap;

// HELPER FUNCTIONS
// Type alias for the standard EffectEthos used in tests
pub(in crate::types::telos_types::effect_ethos) type TestEthos = EffectEthos<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

// Predicate that always returns true
pub(in crate::types::telos_types::effect_ethos) fn always_true_predicate(
    _context: &BaseContext,
    _action: &ProposedAction,
) -> bool {
    true
}

// Predicate that always returns false
pub(in crate::types::telos_types::effect_ethos) fn always_false_predicate(
    _context: &BaseContext,
    _action: &ProposedAction,
) -> bool {
    false
}

// Predicate that checks if a "speed" parameter is greater than 50.0
pub(in crate::types::telos_types::effect_ethos) fn check_speed_predicate(
    _context: &BaseContext,
    action: &ProposedAction,
) -> bool {
    if let Some(ActionParameterValue::Number(speed)) = action.parameters().get("speed") {
        *speed > 50.0
    } else {
        false
    }
}

// Predicate that always returns Uncertain(true)
pub(in crate::types::telos_types::effect_ethos) fn always_uncertain_predicate(
    _context: &BaseContext,
    _action: &ProposedAction,
) -> Result<Uncertain<bool>, CausalityError> {
    Ok(Uncertain::<bool>::point(true))
}

// Creates a dummy context for testing
pub(in crate::types::telos_types::effect_ethos) fn get_dummy_context() -> BaseContext {
    BaseContext::with_capacity(0, "dummy_context", 10)
}

// Creates a dummy action for testing predicates
pub(in crate::types::telos_types::effect_ethos) fn get_dummy_action(
    action_name: &str,
    speed: f64,
) -> ProposedAction {
    let mut params = HashMap::new();
    params.insert("speed".to_string(), ActionParameterValue::Number(speed));
    ProposedAction::new(0, action_name.to_string(), params)
}
