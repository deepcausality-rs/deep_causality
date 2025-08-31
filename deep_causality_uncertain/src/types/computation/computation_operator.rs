/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Defines binary arithmetic operations that take two f64 and return an f64.
#[derive(Debug, Clone, Copy)]
pub enum ArithmeticOperator {
    Add,
    Sub,
    Mul,
    Div,
}

impl ArithmeticOperator {
    pub fn apply(&self, a: f64, b: f64) -> f64 {
        match self {
            ArithmeticOperator::Add => a + b,
            ArithmeticOperator::Sub => a - b,
            ArithmeticOperator::Mul => a * b,
            ArithmeticOperator::Div => a / b,
        }
    }
}

/// Defines binary comparison operations that take an f64 and return a bool.
#[derive(Debug, Clone, Copy)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    EqualTo,
}

impl ComparisonOperator {
    // Note: `apply` here is against a constant threshold, a common case.
    pub fn apply(&self, a: f64, b: f64) -> bool {
        match self {
            ComparisonOperator::GreaterThan => a > b,
            ComparisonOperator::LessThan => a < b,
            // Use a small epsilon for robust floating-point equality checks.
            ComparisonOperator::EqualTo => (a - b).abs() < f64::EPSILON,
        }
    }
}

/// Defines logical operations that take bool(s) and return a bool.
#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    And,
    Or,
    Not, // Note: `Not` is unary, which will be handled by the sampler logic.
}
