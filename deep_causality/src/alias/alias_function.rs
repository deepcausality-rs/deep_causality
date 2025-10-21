/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AssumptionError, CausalityError, Context, PropagatingEffect};
use std::sync::Arc;

// Fn aliases for assumable, assumption, & assumption collection
/// Function type for evaluating numerical values and returning a boolean result.
/// This remains unchanged as it serves a different purpose outside the core causal reasoning.
pub type EvalFn = fn(&[PropagatingEffect]) -> Result<bool, AssumptionError>;

/// The unified function signature for all singleton causaloids that do not require an external context.
///
/// This function is a core part of the reasoning engine.
///
/// # Arguments
///
/// * `effect` - A reference to the `PropagatingEffect` flowing through the graph during reasoning.
///
/// # Returns
///
/// A `Result` containing either a `PropagatingEffect` on success or a `CausalityError` on failure.
pub type CausalFn = fn(effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError>;

/// The unified function signature for all singleton causaloids that require access to a shared, external context.
///
/// It evaluates runtime evidence against its own static configuration and the shared context
/// to produce a causal effect.
///
/// # Arguments
///
/// * `effect` - A reference to the `PropagatingEffect` flowing through the graph during reasoning.
/// * `context` - A reference to the shared `Context` object.
///
/// # Returns
///
/// A `Result` containing either a `PropagatingEffect` on success or a `CausalityError` on failure.
pub type ContextualCausalFn<D, S, T, ST, SYM, VS, VT> =
    fn(
        effect: &PropagatingEffect,
        context: &Arc<Context<D, S, T, ST, SYM, VS, VT>>,
    ) -> Result<PropagatingEffect, CausalityError>;
