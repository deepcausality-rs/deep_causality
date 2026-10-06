/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the distillation-round example.

/// The index of the noise world the cross-precision comparison uses: the largest probability, where
/// the residual is furthest from zero and a difference between scalars has room to show.
pub const COMPARISON_INDEX: usize = 2;
