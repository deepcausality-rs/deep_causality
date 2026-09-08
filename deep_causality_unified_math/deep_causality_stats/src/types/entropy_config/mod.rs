/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The three parameters an information measure needs.

use crate::types::log_base::LogBase;
use crate::types::normalisation::Normalisation;
use crate::types::zero_policy::ZeroPolicy;

/// Base, zero policy and normalisation, together.
///
/// The three axes are independent and the workspace's shipped implementations disagree on all
/// three at once, so they travel together rather than as three positional arguments. Each
/// combination named here has a caller; none is speculative.
///
/// The default is bits, skip-at-zero, no normalisation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EntropyConfig<T> {
    /// The unit of the answer.
    pub base: LogBase,
    /// Where the cutoff for a negligible entry sits.
    pub zero_policy: ZeroPolicy<T>,
    /// Whether the input is divided by its sum first.
    pub normalisation: Normalisation<T>,
}

impl<T> EntropyConfig<T> {
    /// Bits, skipping exactly zero, taking the input as a distribution.
    pub fn bits() -> Self
    where
        T: Default,
    {
        Self {
            base: LogBase::Bits,
            zero_policy: ZeroPolicy::SkipZero,
            normalisation: Normalisation::None,
        }
    }

    /// Nats, skipping exactly zero, taking the input as a distribution.
    pub fn nats() -> Self
    where
        T: Default,
    {
        Self {
            base: LogBase::Nats,
            zero_policy: ZeroPolicy::SkipZero,
            normalisation: Normalisation::None,
        }
    }

    /// Replaces the base.
    pub fn with_base(mut self, base: LogBase) -> Self {
        self.base = base;
        self
    }

    /// Replaces the zero policy.
    pub fn with_zero_policy(mut self, zero_policy: ZeroPolicy<T>) -> Self {
        self.zero_policy = zero_policy;
        self
    }

    /// Replaces the normalisation.
    pub fn with_normalisation(mut self, normalisation: Normalisation<T>) -> Self {
        self.normalisation = normalisation;
        self
    }
}
