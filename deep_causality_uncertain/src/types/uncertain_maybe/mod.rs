/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod uncertain_maybe_bool;
mod uncertain_maybe_f64;
mod uncertain_maybe_lift;
mod uncertain_maybe_op_arithmetic;

use crate::Uncertain;

/// A first-class type representing a value that is probabilistically present or absent.
/// If the value is present, its own value is uncertain.
#[derive(Debug, Clone, PartialEq)]
pub struct MaybeUncertain<T> {
    is_present: Uncertain<bool>,
    value: Uncertain<T>,
}
