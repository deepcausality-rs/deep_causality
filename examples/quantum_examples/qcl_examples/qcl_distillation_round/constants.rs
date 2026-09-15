/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the distillation-round example.

/// The depolarising probabilities swept on every physical qubit of the encoded magic-state
/// preparation; zero is the noiseless round.
pub const NOISE_SWEEP: [f64; 3] = [0.0, 0.01, 0.05];
