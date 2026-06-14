/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::RealField;

/// Defines binary arithmetic operations on any real scalar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArithmeticOperator {
    Add,
    Sub,
    Mul,
    Div,
}

impl ArithmeticOperator {
    /// Apply the operation at the operands' precision. Generic over `R: RealField`
    /// (`Add`/`Sub`/`Mul` from the ring, `Div` from the field), so the same code path
    /// serves `f64` (bit-identically) and `Float106`.
    pub fn apply<R: RealField>(&self, a: R, b: R) -> R {
        match self {
            ArithmeticOperator::Add => a + b,
            ArithmeticOperator::Sub => a - b,
            ArithmeticOperator::Mul => a * b,
            ArithmeticOperator::Div => a / b,
        }
    }
}

use std::fmt;

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArithmeticOperator::Add => write!(f, "Add"),
            ArithmeticOperator::Sub => write!(f, "Sub"),
            ArithmeticOperator::Mul => write!(f, "Mul"),
            ArithmeticOperator::Div => write!(f, "Div"),
        }
    }
}
