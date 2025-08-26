/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod display;

use crate::PropagatingEffect;

/// Represents a value for a parameter within a `ProposedAction`.
/// This allows the parameters map to hold values of different types.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionParameterValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
}

impl From<PropagatingEffect> for ActionParameterValue {
    fn from(effect: PropagatingEffect) -> Self {
        match effect {
            PropagatingEffect::Deterministic(b) => ActionParameterValue::Boolean(b),
            PropagatingEffect::Numerical(n) => ActionParameterValue::Number(n),
            PropagatingEffect::Probabilistic(p) => ActionParameterValue::Number(p),
            // Other variants can be converted to a string representation for logging/debugging.
            _ => ActionParameterValue::String(format!("{:?}", effect)),
        }
    }
}
