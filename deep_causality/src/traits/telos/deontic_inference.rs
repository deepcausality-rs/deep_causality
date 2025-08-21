/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Context, Datable, DeonticError, ProposedAction, SpaceTemporal, Spatial, Symbolic, Temporal,
    Verdict,
};

/// Defines the public API for a deontic reasoning engine.
#[allow(clippy::type_complexity)]
pub trait DeonticInference<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Evaluates a proposed action against the set of norms within a given context.
    ///
    /// # Arguments
    /// * `action` - A reference to the `ProposedAction` being evaluated.
    /// * `context` - A reference to the current `Context` providing the state of the world.
    ///
    /// # Returns
    /// A `Result` containing either:
    /// * `Ok(Verdict)` - A rich `Verdict` struct with the deontic outcome and justification.
    /// * `Err(DeonticError)` - An error indicating why the evaluation could not be completed.
    fn evaluate_action(
        &self,
        action: &ProposedAction,
        context: &Context<D, S, T, ST, SYM, VS, VT>,
    ) -> Result<Verdict, DeonticError>;
}
