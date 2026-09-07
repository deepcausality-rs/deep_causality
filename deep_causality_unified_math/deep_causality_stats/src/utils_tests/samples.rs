/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Sample families the property cases run over.

/// The samples the property cases run over: a sample with repeated values, a short arithmetic run,
/// a sample straddling zero, a two-element sample, and a constant sample. Every one has `n ≥ 2`,
/// so `variance` and `std_dev` are defined on all of them.
pub const FAMILY: [&[f64]; 5] = [
    &[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0],
    &[1.0, 2.0, 3.0, 4.0, 5.0],
    &[-3.0, 1.0, 5.0],
    &[0.5, -0.25],
    &[7.0, 7.0, 7.0, 7.0],
];
