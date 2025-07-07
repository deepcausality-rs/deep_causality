/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::errors::CausalityError;
use crate::{Context, NumericalValue};
use std::sync::Arc;

// Fn aliases for assumable, assumption, & assumption collection
/// Function type for evaluating numerical values and returning a boolean result
pub type EvalFn = fn(&[NumericalValue]) -> bool;

// Fn aliases for causal function with and without context
/// Function type for basic causal operations that returns a Result containing either boolean or CausalityError
pub type CausalFn = fn(&NumericalValue) -> Result<bool, CausalityError>;

/// Function type for causal operations that take a numerical value and context, returning a boolean result or error
pub type ContextualCausalFn<D, S, T, ST, SYM, VS, VT> =
    fn(&NumericalValue, &Arc<Context<D, S, T, ST, SYM, VS, VT>>) -> Result<bool, CausalityError>;
