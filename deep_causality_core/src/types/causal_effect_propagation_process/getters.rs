/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalEffectPropagationProcess, EffectValue};

// Getters
impl<Value, State, Context, Error, Log>
    CausalEffectPropagationProcess<Value, State, Context, Error, Log>
{
    pub fn value(&self) -> &EffectValue<Value> {
        &self.value
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn context(&self) -> &Option<Context> {
        &self.context
    }

    pub fn error(&self) -> &Option<Error> {
        &self.error
    }

    pub fn logs(&self) -> &Log {
        &self.logs
    }
}
