/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::errors::CausalityError;
use crate::prelude::{Context, NumericalValue, SymbolicResult};
use std::sync::Arc;

// Fn aliases for assumable, assumption, & assumption collection
pub type EvalFn = fn(&[NumericalValue]) -> bool;

pub type SymbolicRepr = String;
pub type SymbolicCausalFn = fn(SymbolicRepr) -> Result<SymbolicResult, CausalityError>;

pub type ProbabilisticCausalFn = fn(NumericalValue) -> Result<NumericalValue, CausalityError>; // Bayes update, etc.

// Fn aliases for causal function with and without context
pub type CausalFn = fn(&NumericalValue) -> Result<bool, CausalityError>;

pub type ContextualCausalDataFn<D, S, T, ST, SYM, VS, VT> =
    fn(&NumericalValue, &Arc<Context<D, S, T, ST, SYM, VS, VT>>) -> Result<bool, CausalityError>;

pub type ContextualCausalFn<D, S, T, ST, SYM, VS, VT> =
    fn(&Arc<Context<D, S, T, ST, SYM, VS, VT>>) -> Result<bool, CausalityError>;
