/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    CausalEffectLog, CausalPropagatingEffect, CausalityError, EffectValue, NumericalValue,
    PropagatingEffect,
};

// f(U_smoking) -> Smoking
pub fn smoking_logic(nicotine_obs: EffectValue) -> PropagatingEffect {
    let mut log = CausalEffectLog::new();
    // Propagates an error if nicotine_obs is not a Numerical value.
    if let Some(nicotine_level) = nicotine_obs.as_numerical() {
        let threshold: NumericalValue = 0.6;
        let high_nicotine = nicotine_level > &threshold;
        log.add_entry(&format!(
            "Nicotine level {} is higher than threshold {}: {}",
            nicotine_obs, threshold, high_nicotine
        ));
        CausalPropagatingEffect::from_effect_value_with_log(
            EffectValue::Boolean(high_nicotine),
            log,
        )
    } else {
        PropagatingEffect::from_error(CausalityError::new(
            "Expected Numerical value for smoking_logic".to_string(),
        ))
    }
}

// f(Smoking) -> Tar
pub fn tar_logic(is_smoking: EffectValue) -> PropagatingEffect {
    let mut log = CausalEffectLog::new();
    if let Some(has_tar) = is_smoking.as_bool() {
        log.add_entry(&format!("Has tar in lung {}", has_tar));
        CausalPropagatingEffect::from_effect_value_with_log(EffectValue::Boolean(has_tar), log)
    } else {
        let err = CausalityError::new("Expected Boolean value for tar_logic".to_string());
        PropagatingEffect::from_error(err)
    }
}

pub fn error_logic(
    input: EffectValue,
) -> CausalPropagatingEffect<EffectValue, CausalityError, CausalEffectLog> {
    let mut log = CausalEffectLog::new();
    log.add_entry("Error logic applied");
    if input.as_bool().unwrap_or(false) {
        CausalPropagatingEffect {
            value: EffectValue::None,
            error: Some(CausalityError::new("Simulated error".to_string())),
            logs: log,
        }
    } else {
        CausalPropagatingEffect::from_effect_value_with_log(input, log)
    }
}
