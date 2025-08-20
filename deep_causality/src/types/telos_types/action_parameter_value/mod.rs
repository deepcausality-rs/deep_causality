/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::{Display, Formatter};

/// Represents a value for a parameter within a `ProposedAction`.
/// This allows the parameters map to hold values of different types.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionParameterValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
}

impl Display for ActionParameterValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionParameterValue::String(s) => write!(f, "{}", s),
            ActionParameterValue::Number(n) => write!(f, "{}", n),
            ActionParameterValue::Integer(i) => write!(f, "{}", i),
            ActionParameterValue::Boolean(b) => write!(f, "{}", b),
        }
    }
}
