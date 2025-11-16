/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, EffectValue};
use std::fmt::{Display, Formatter};

/// Represents a value for a parameter within a `ProposedAction`.
/// This allows the parameters map to hold values of different types.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionParameterValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    /// A link to a complex, structured result in a Contextoid. As an input, this
    /// can be interpreted as a command to fetch data from the context using the ID's.
    ContextualLink(ContextoidId, ContextoidId),
}

impl From<EffectValue> for ActionParameterValue {
    fn from(effect_value: EffectValue) -> Self {
        match effect_value {
            EffectValue::Deterministic(b) => ActionParameterValue::Boolean(b),
            EffectValue::Numerical(n) => ActionParameterValue::Number(n),
            EffectValue::Probabilistic(p) => ActionParameterValue::Number(p),
            EffectValue::UncertainBool(u) => ActionParameterValue::Boolean(u.value()),
            EffectValue::UncertainFloat(u) => ActionParameterValue::Number(u.value()),
            EffectValue::Tensor(t) => ActionParameterValue::String(format!("{:?}", t)),
            EffectValue::Complex(c) => ActionParameterValue::String(format!("{:?}", c)),
            EffectValue::ContextualLink(context_id, contextoid_id) => {
                ActionParameterValue::ContextualLink(context_id, contextoid_id)
            }
            EffectValue::None => ActionParameterValue::String("None".to_string()),
            _ => ActionParameterValue::String("Unsupported EffectValue".to_string()),
        }
    }
}

impl Display for ActionParameterValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionParameterValue::String(s) => write!(f, "ActionParameterValue::String: {}", s),
            ActionParameterValue::Number(n) => write!(f, "ActionParameterValue::Number: {:.2}", n),
            ActionParameterValue::Integer(i) => write!(f, "ActionParameterValue::Integer: {}", i),
            ActionParameterValue::Boolean(b) => write!(f, "ActionParameterValue::Boolean: {}", b),
            ActionParameterValue::ContextualLink(context_id, contextoid_id) => {
                write!(
                    f,
                    "ActionParameterValue::ContextualLink({}, {})",
                    context_id, contextoid_id
                )
            }
        }
    }
}
