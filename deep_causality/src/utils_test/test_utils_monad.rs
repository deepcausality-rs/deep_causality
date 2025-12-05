/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalEffectLog, CausalMonad, CausalityError, CausalityErrorEnum, PropagatingEffect};
use deep_causality_haft::{LogAddEntry, MonadEffect5};

// f(U_smoking) -> Smoking
// Input: Nicotine level (f64)
// Output: High Nicotine (bool)
pub fn smoking_logic(nicotine_obs: f64) -> PropagatingEffect<bool> {
    let mut log = CausalEffectLog::new();
    let threshold = 0.6;
    let high_nicotine = nicotine_obs > threshold;
    log.add_entry(&format!(
        "Nicotine level {} is higher than threshold {}: {}",
        nicotine_obs, threshold, high_nicotine
    ));

    let mut effect = CausalMonad::pure(high_nicotine);
    effect.logs = log;
    effect
}

// f(Smoking) -> Tar
// Input: Is Smoking (bool)
// Output: Has Tar (bool)
pub fn tar_logic(is_smoking: bool) -> PropagatingEffect<bool> {
    let mut log = CausalEffectLog::new();
    log.add_entry(&format!("Has tar in lung {}", is_smoking));

    let mut effect = CausalMonad::pure(is_smoking);
    effect.logs = log;
    effect
}

pub fn error_logic() -> PropagatingEffect<()> {
    let mut log = CausalEffectLog::new();
    log.add_entry("Error logic applied");

    let mut effect = PropagatingEffect::from_error(CausalityError::new(
        CausalityErrorEnum::Custom("Simulated error".to_string()),
    ));
    effect.logs = log;
    effect
}
