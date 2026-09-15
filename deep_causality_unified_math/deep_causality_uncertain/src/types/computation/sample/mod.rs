/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What one evaluation of a node produces.

use crate::UncertainError;
use core::fmt::{Display, Formatter};

/// The value a node yields for one draw: a real at the graph's scalar, or a Boolean.
///
/// # Why two variants and not one
///
/// The scalar is a parameter of the graph, so a real-valued node needs no variant to say which
/// precision it carries — `R` says it. What the scalar cannot say is that a node is a *Boolean*:
/// a Bernoulli leaf, a comparison and a logical combination all live in a tree of reals and
/// produce a truth value. Those two cases are the whole of the enum, and neither is a precision.
///
/// This replaces the closed three-variant dispatcher (`Float` / `DoubleFloat` / `Bool`) that named
/// its precisions. Adding a scalar to that enum meant adding a variant and every match arm reached
/// by it; adding a scalar here means nothing, which is the difference the retrofit is for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sample<R> {
    /// A real value at the graph's scalar.
    Real(R),
    /// A truth value.
    Bool(bool),
}

impl<R> Sample<R> {
    /// The real value, or a typed error naming what was found instead.
    ///
    /// The two accessors are how a carrier reads its own kind off a shared graph: `Uncertain<R>`
    /// calls [`Self::real`], `UncertainBool<R>` calls [`Self::boolean`], and a graph whose root
    /// disagrees with the carrier reports rather than substituting a default.
    pub(crate) fn real(self) -> Result<R, UncertainError> {
        match self {
            Sample::Real(value) => Ok(value),
            Sample::Bool(_) => Err(UncertainError::UnsupportedTypeError(
                "expected a real sample, found a Boolean one".to_string(),
            )),
        }
    }

    /// The truth value, or a typed error naming what was found instead.
    pub(crate) fn boolean(self) -> Result<bool, UncertainError> {
        match self {
            Sample::Bool(value) => Ok(value),
            Sample::Real(_) => Err(UncertainError::UnsupportedTypeError(
                "expected a Boolean sample, found a real one".to_string(),
            )),
        }
    }
}

impl<R: Display> Display for Sample<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Sample::Real(value) => write!(f, "{}", value),
            Sample::Bool(value) => write!(f, "{}", value),
        }
    }
}
