// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::types::alias_types::NumericalValue;

pub const ZERO: NumericalValue = 0.0;
pub const MINUS_ONE: NumericalValue = -1.0;

/// returns the absolute value of a numerical value
pub fn abs_num(val: NumericalValue) -> NumericalValue {
    if val > ZERO { val } else { MINUS_ONE * val }
}
