/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::UncertainBool;
use deep_causality_rand::RandScalar;

/// Certainly true — the top of the Boolean verdict lattice.
impl<R: RandScalar> Default for UncertainBool<R> {
    fn default() -> Self {
        Self::point(true)
    }
}
