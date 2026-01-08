/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Display implementations for Manifold.

//! Display trait implementation for Manifold.

use crate::Manifold;
use core::fmt;
use std::fmt::Formatter;

impl<C, D> fmt::Display for Manifold<C, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Manifold {{ dimension: {}, simplices: {} }}",
            self.complex.skeletons.last().map(|s| s.dim).unwrap_or(0),
            self.complex
                .skeletons
                .iter()
                .map(|s| s.simplices.len())
                .sum::<usize>()
        )
    }
}
