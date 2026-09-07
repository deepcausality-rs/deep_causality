/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Whether an information measure normalises its input first.

/// What to do when the input does not sum to one.
///
/// A parameter because the shipped implementations differ: one takes its input as a distribution
/// already, the other divides by the sum of its present entries because its input is a marginal
/// over optional values and the absent ones carry no mass.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Normalisation<T> {
    /// Use the input as given. It is the caller's statement that it sums to one.
    #[default]
    None,
    /// Divide by the sum of the entries first.
    ///
    /// When that sum is at or below `floor` the input carries no mass, there is no distribution
    /// to measure, and the result is zero rather than a division by something near zero.
    BySum { floor: T },
}
