/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NumericValue;
use core::fmt::{Display, Formatter};

impl Display for NumericValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericValue::None => write!(f, "None"),
            NumericValue::U8(n) => write!(f, "U8({})", n),
            NumericValue::U16(n) => write!(f, "U16({})", n),
            NumericValue::U32(n) => write!(f, "U32({})", n),
            NumericValue::U64(n) => write!(f, "U64({})", n),
            NumericValue::U128(n) => write!(f, "U128({})", n),
            NumericValue::I8(n) => write!(f, "I8({})", n),
            NumericValue::I16(n) => write!(f, "I16({})", n),
            NumericValue::I32(n) => write!(f, "I32({})", n),
            NumericValue::I64(n) => write!(f, "I64({})", n),
            NumericValue::I128(n) => write!(f, "I128({})", n),
            NumericValue::F32(n) => write!(f, "F32({})", n),
            NumericValue::F64(n) => write!(f, "F64({})", n),
        }
    }
}
