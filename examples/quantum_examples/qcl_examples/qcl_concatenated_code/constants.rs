/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the concatenated-code example.

/// The residual below which a square is read as exact at `f64`; the program reports the measured
/// values and this line at every precision it runs.
pub const EXACT_AT_F64: f64 = 1e-9;
