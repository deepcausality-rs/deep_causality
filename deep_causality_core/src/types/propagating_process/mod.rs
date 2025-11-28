/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
pub mod hkt;

use crate::EffectLog;
use crate::errors::causality_error::CausalityError;
use crate::types::causal_effect_propagation_process::CausalEffectPropagationProcess;

pub type PropagatingProcess<T, S, C> =
    CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>;
