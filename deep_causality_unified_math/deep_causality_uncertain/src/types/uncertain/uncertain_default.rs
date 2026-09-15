/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Uncertain;
use crate::UncertainScalar;

/// The certain zero of the scalar.
///
/// One impl at every scalar rather than one per precision. The value is `R::zero()` — the additive
/// identity the algebra already supplies — rather than a literal, which is what lets the impl exist
/// for a scalar this crate does not name.
impl<R: UncertainScalar> Default for Uncertain<R> {
    fn default() -> Self {
        Self::point(R::zero())
    }
}
