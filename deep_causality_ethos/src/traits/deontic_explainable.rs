/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{DeonticError, Verdict};
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};
/// A trait for explaining the reasoning behind a deontic verdict.
#[allow(clippy::type_complexity)]
pub trait DeonticExplainable<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    /// Explains the reasoning behind a given verdict.
    ///
    /// # Arguments
    /// * `verdict` - A reference to the `Verdict` to be explained.
    ///
    /// # Returns
    /// A `Result` containing a `String` with a human-readable explanation,
    /// or a `DeonticError` if the explanation cannot be generated.
    fn explain_verdict(&self, verdict: &Verdict) -> Result<String, DeonticError>;
}
