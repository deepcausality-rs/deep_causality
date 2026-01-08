/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::CausalEffectPropagationProcess;
use crate::EffectValue;
use crate::Intervenable;
use alloc::format;
use core::fmt::Debug;
use deep_causality_haft::{LogAddEntry, LogAppend};

impl<Value, State, Context, Error, Log> Intervenable<Value>
    for CausalEffectPropagationProcess<Value, State, Context, Error, Log>
where
    Log: LogAppend + LogAddEntry + Default,
    Value: Debug,
{
    fn intervene(mut self, new_value: Value) -> Self {
        // If there is already an error, we propagate it and do nothing else.
        if self.error.is_some() {
            return self;
        }

        let new_value = EffectValue::from(new_value);

        self.logs.add_entry(&format!(
            "!!Intervention!!: {:?} replaced with {:?}",
            self.value, new_value
        ));
        self.value = new_value;
        self
    }
}
