/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    CausalEffectLog, CausalPropagatingEffect, CausalityError, EffectValue, NumericalValue,
};

// Helper functions from the test file, adapted for benchmarking
pub fn smoking_logic(
    nicotine_obs: EffectValue,
) -> CausalPropagatingEffect<EffectValue, CausalityError, CausalEffectLog> {
    let mut log = CausalEffectLog::new();
    let nicotine_level = nicotine_obs.as_numerical().unwrap_or(&0.0);
    let threshold: NumericalValue = 0.6;
    let high_nicotine = nicotine_level > &threshold;
    log.add_entry(&format!(
        "Nicotine level {} is higher than threshold {}: {}",
        nicotine_obs, threshold, high_nicotine
    ));
    CausalPropagatingEffect::from_effect_value_with_log(EffectValue::Boolean(high_nicotine), log)
}

pub fn tar_logic(
    is_smoking: EffectValue,
) -> CausalPropagatingEffect<EffectValue, CausalityError, CausalEffectLog> {
    let mut log = CausalEffectLog::new();
    let has_tar = is_smoking.as_bool().unwrap_or(false);
    log.add_entry(&format!("Has tar in lung {}", has_tar));
    CausalPropagatingEffect::from_effect_value_with_log(EffectValue::Boolean(has_tar), log)
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
