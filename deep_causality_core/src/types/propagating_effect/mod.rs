// SPDX-License-Identifier: MIT
// Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.

use crate::types::causal_effect_propagation_process::CausalEffectPropagationProcess;
use crate::{CausalityError, EffectLog};

pub mod hkt;

pub type PropagatingEffect<T> =
    CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;
