/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the code-switching example.

/// The depolarising probability of the noisy gadget, applied to the first logical wire between
/// the decoder of code A and the encoder of code B.
pub const GADGET_NOISE: f64 = 0.1;

/// The side of the square torus giving code B, the `[[8,2,2]]` toric code.
pub const TORUS_SIDE: usize = 2;
