/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Defines the `intervene` operation for a monadic effect system.
/// This trait is intended for causal reasoning systems where counterfactuals
/// are modeled by forcing a value at a specific point in a computation chain.
pub trait Intervenable<T> {
    /// Overrides the value within an effectful computation.
    ///
    /// This function takes an existing `effect` (self) and a `new_value`. It returns a new
    /// effect where the original value is discarded and replaced by `new_value`.
    ///
    /// Crucially, it should preserve the context of the computation:
    /// - **Error State**: If the incoming `effect` was already in an error state,
    ///   that error is propagated. An intervention cannot fix a previously broken chain.
    /// - **Log History**: The logs from the incoming `effect` are preserved, and a
    ///   new entry is added to signify that an intervention occurred.
    fn intervene(self, new_value: T) -> Self;
}
