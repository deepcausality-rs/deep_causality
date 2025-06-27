/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::errors::CausalityError;
use crate::prelude::{Context, NumericalValue, SymbolicResult};
use std::sync::Arc;

// Fn aliases for assumable, assumption, & assumption collection
/// Function type for evaluating numerical values and returning a boolean result
pub type EvalFn = fn(&[NumericalValue]) -> bool;

/// Type alias for symbolic representation using String
pub type SymbolicRepr = String;
/// Function type for symbolic causal operations that returns a Result containing either SymbolicResult or CausalityError
pub type SymbolicCausalFn = fn(SymbolicRepr) -> Result<SymbolicResult, CausalityError>;

/// Function type for probabilistic causal operations like Bayes updates that returns a Result containing either NumericalValue or CausalityError
pub type ProbabilisticCausalFn = fn(NumericalValue) -> Result<NumericalValue, CausalityError>; // Bayes update, etc.

// Fn aliases for causal function with and without context
/// Function type for basic causal operations that returns a Result containing either boolean or CausalityError
pub type CausalFn = fn(&NumericalValue) -> Result<bool, CausalityError>;

/// Function type for causal operations that take a numerical value and context, returning a boolean result or error
pub type ContextualCausalDataFn<D, S, T, ST, SYM, VS, VT> =
    fn(&NumericalValue, &Arc<Context<D, S, T, ST, SYM, VS, VT>>) -> Result<bool, CausalityError>;

/// Function type for causal operations that only take a context, returning a boolean result or error
pub type ContextualCausalFn<D, S, T, ST, SYM, VS, VT> =
    fn(&Arc<Context<D, S, T, ST, SYM, VS, VT>>) -> Result<bool, CausalityError>;
