/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod display;

/// Represents a value for a parameter within a `ProposedAction`.
/// This allows the parameters map to hold values of different types.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionParameterValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
}
