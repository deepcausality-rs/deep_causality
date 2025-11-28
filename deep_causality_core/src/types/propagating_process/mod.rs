/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod constructors;
mod display;
pub mod hkt;

use crate::EffectValue;

#[derive(Debug, PartialEq, Clone)]
pub struct CausalProcessEffect<Value, State, Context, Error, Log> {
    pub value: EffectValue<Value>,
    pub state: State,
    pub context: Option<Context>,
    pub error: Option<Error>,
    pub logs: Log,
}

pub type ProcessEffect<Value, State, Context, Error, Log> =
    CausalProcessEffect<Value, State, Context, Error, Log>;
