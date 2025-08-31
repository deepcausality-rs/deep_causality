/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Defines the arithmetic operations that can be represented in the graph.
#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Mul,
}

impl Operator {
    pub fn apply(&self, a: f64, b: f64) -> f64 {
        match self {
            Operator::Add => a + b,
            Operator::Mul => a * b,
        }
    }
}
