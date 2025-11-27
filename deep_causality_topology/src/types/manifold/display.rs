/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Manifold;
use core::fmt;
use core::fmt::Formatter;

impl<T> fmt::Display for Manifold<T> {
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
