/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{DeonticError, Verdict};
use deep_causality::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
/// A trait for explaining the reasoning behind a deontic verdict.
#[allow(clippy::type_complexity)]
pub trait DeonticExplainable<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
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
