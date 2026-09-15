/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::ToPrimitive;

/// Struct to hold the parameters for a Bernoulli distribution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BernoulliParams {
    pub p: f64, // probability of success
}

impl BernoulliParams {
    /// Creates a new `BernoulliParams` instance.
    ///
    /// # Arguments
    ///
    /// * `p` - The probability of success (must be between 0.0 and 1.0, inclusive).
    ///
    /// # Returns
    ///
    /// A new `BernoulliParams` instance.
    pub fn new(p: f64) -> Self {
        Self { p }
    }

    /// The same parameter, stated in any scalar.
    ///
    /// The field stays 64-bit because that is what the draw honours: `Bernoulli::new` holds `p` as
    /// 64-bit fixed point, which is what makes `p = 0` and `p = 1` exact. A caller stating a
    /// probability at a wider scalar keeps 64 bits of it, and one stating it at a narrower scalar
    /// loses nothing here — the loss already happened when the narrow scalar held the literal.
    /// This is a bound of the representation rather than of the scalar, and it is recorded at
    /// [`UncertainBool::bernoulli`](crate::UncertainBool::bernoulli) where a caller meets it.
    ///
    /// A scalar that cannot state `p` at all — a `NaN` — lands as `NaN`, and `Bernoulli::new`
    /// rejects it there rather than here, with the message that names the parameter.
    pub fn at<R: ToPrimitive>(p: R) -> Self {
        Self {
            p: p.to_f64().unwrap_or(f64::NAN),
        }
    }
}

impl std::fmt::Display for BernoulliParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BernoulliParams {{ p: {:.2} }}", self.p)
    }
}
