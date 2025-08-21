/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    ActionParameterValue, BaseContext, BaseSymbol, Data, EffectEthos, EuclideanSpace,
    EuclideanSpacetime, EuclideanTime, FloatType, NumericalValue, ProposedAction, Teloid, TeloidID,
    TeloidModal,
};
use std::collections::HashMap;

fn always_true_predicate(_context: &BaseContext, _action: &ProposedAction) -> bool {
    true
}

fn always_false_predicate(_context: &BaseContext, _action: &ProposedAction) -> bool {
    false
}

fn check_speed_predicate(_context: &BaseContext, action: &ProposedAction) -> bool {
    if let Some(ActionParameterValue::Number(speed)) = action.parameters().get("speed") {
        *speed > 50.0
    } else {
        false
    }
}

fn get_dummy_context() -> BaseContext {
    BaseContext::with_capacity(0, "dummy_context", 1)
}

fn get_dummy_action(speed: f64) -> ProposedAction {
    let mut params = HashMap::new();
    params.insert("speed".to_string(), ActionParameterValue::Number(speed));
    ProposedAction::new(0, "test_action".to_string(), params)
}

fn create_test_teloid(
    id: TeloidID,
    action_id: String,
    predicate: fn(&BaseContext, &ProposedAction) -> bool,
    modality: TeloidModal,
    timestamp: u64,
    specificity: u32,
) -> Teloid<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
> {
    Teloid::new(
        id,
        action_id,
        predicate,
        modality,
        timestamp,
        specificity,
        5, // Default priority
        vec!["test_tag"],
        None,
    )
}

#[test]
fn test_effect_ethos_new() {
    let ethos = EffectEthos::<
        Data<NumericalValue>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >::new();
    assert!(!ethos.is_verified());
}
