/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ActionParameterValue;
use std::fmt::{Display, Formatter};

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
