// SPDX-License-Identifier: MIT
// Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.

use deep_causality::{
    CausalEffectLog, CausalPropagatingEffect, EffectValue, NumericalValue, PropagatingEffect,
};

// Each function represents a structural equation in an SCM.
// They take a value and return a new PropagatingEffect.

// f(U_smoking) -> Smoking
pub fn smoking_logic(nicotine_obs: EffectValue) -> PropagatingEffect {
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

// f(Smoking) -> Tar
pub fn tar_logic(is_smoking: EffectValue) -> PropagatingEffect {
    let mut log = CausalEffectLog::new();
    let has_tar = is_smoking.as_bool().unwrap_or(false);
    log.add_entry(&format!("Has tar in lung {}", has_tar));
    CausalPropagatingEffect::from_effect_value_with_log(EffectValue::Boolean(has_tar), log)
}

// f(Tar, GeneticPredisposition) -> Cancer
// This function now includes an unobserved background variable (exogenous factor).
pub fn cancer_logic(has_tar: EffectValue, has_genetic_predisposition: bool) -> PropagatingEffect {
    let mut log = CausalEffectLog::new();
    log.add_entry(&format!("Has tar in lung {}", has_tar));
    log.add_entry(&format!("Has genetic risk {}", has_genetic_predisposition));
    let has_cancer_risk = has_tar.as_bool().unwrap_or(false) || has_genetic_predisposition;
    log.add_entry(&format!("Has cancer risk {}", has_cancer_risk));
    CausalPropagatingEffect::from_effect_value_with_log(EffectValue::Boolean(has_cancer_risk), log)
}
