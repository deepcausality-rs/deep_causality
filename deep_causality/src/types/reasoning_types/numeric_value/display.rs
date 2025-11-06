/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NumericValue;
use std::fmt::{Display, Formatter};

impl Display for NumericValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericValue::None => write!(f, "None"),
            NumericValue::U8(n) => write!(f, "{}", n),
            NumericValue::U16(n) => write!(f, "{}", n),
            NumericValue::U32(n) => write!(f, "{}", n),
            NumericValue::U64(n) => write!(f, "{}", n),
            NumericValue::U128(n) => write!(f, "{}", n),
            NumericValue::I8(n) => write!(f, "{}", n),
            NumericValue::I16(n) => write!(f, "{}", n),
            NumericValue::I32(n) => write!(f, "{}", n),
            NumericValue::I64(n) => write!(f, "{}", n),
            NumericValue::I128(n) => write!(f, "{}", n),
            NumericValue::F32(n) => write!(f, "{}", n),
            NumericValue::F64(n) => write!(f, "{}", n),
        }
    }
}
