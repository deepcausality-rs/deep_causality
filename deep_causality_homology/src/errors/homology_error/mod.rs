/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The error type of the chain-complex layer.

use alloc::string::{String, ToString};
use core::fmt;
use deep_causality_linear::LinearError;

/// What can go wrong computing homology.
///
/// Two variants, because there are two ways to fail here and they have different causes. A rank can
/// overflow, which is a property of the matrix and the carrier. Two chains can disagree about the
/// group they live in, which is a mistake at the call site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomologyErrorEnum {
    /// The exact elimination overflowed, or a packed operation was given a bad shape.
    ///
    /// Only the characteristic-zero path can overflow: the fraction-free intermediates are minors
    /// of the whole matrix, so they grow with it, and reporting the overflow is what keeps a
    /// wrapped intermediate from being returned as a rank.
    LinearAlgebraError(String),
    /// Two chains do not live in the same chain group.
    ///
    /// `C_k` is identified by the pair `(degree, len)`, and both halves are reported here so a
    /// length mismatch and a degree mismatch surface the same way.
    ChainGroupMismatch(String),
}

/// The crate's error, wrapping [`HomologyErrorEnum`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomologyError(pub HomologyErrorEnum);

impl fmt::Display for HomologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            HomologyErrorEnum::LinearAlgebraError(msg) => write!(f, "Linear algebra error: {msg}"),
            HomologyErrorEnum::ChainGroupMismatch(msg) => write!(f, "Chain group mismatch: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HomologyError {}

impl From<LinearError> for HomologyError {
    fn from(err: LinearError) -> Self {
        HomologyError(HomologyErrorEnum::LinearAlgebraError(err.to_string()))
    }
}

impl HomologyError {
    /// A linear-algebra failure, named.
    #[allow(non_snake_case)]
    pub fn LinearAlgebraError<S: Into<String>>(msg: S) -> Self {
        Self(HomologyErrorEnum::LinearAlgebraError(msg.into()))
    }

    /// Two chains in different chain groups.
    #[allow(non_snake_case)]
    pub fn ChainGroupMismatch<S: Into<String>>(msg: S) -> Self {
        Self(HomologyErrorEnum::ChainGroupMismatch(msg.into()))
    }
}
