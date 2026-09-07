/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The base an information measure is reported in.

/// The logarithm base, and therefore the unit of the answer.
///
/// This is a parameter because the workspace's shipped entropy implementations disagree on it:
/// the causal-discovery paths compute in bits, the thermodynamics kernel in nats. The two differ
/// by a factor of `ln 2`, which is a different number, not a rounding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LogBase {
    /// Base 2. The unit is the bit (the shannon).
    #[default]
    Bits,
    /// Base e. The unit is the nat.
    Nats,
}
