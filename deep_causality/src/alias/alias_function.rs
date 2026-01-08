/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::AssumptionError;
use deep_causality_core::{EffectValue, PropagatingEffect, PropagatingProcess};

// Fn aliases for assumable, assumption, & assumption collection
/// Function type for evaluating numerical values and returning a boolean result.
/// This remains unchanged as it serves a different purpose outside the core causal reasoning.
pub type EvalFn = fn(&[PropagatingEffect<f64>]) -> Result<bool, AssumptionError>;

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
/// A `PropagatingEffect`
pub type CausalFn<I, O> = fn(I) -> PropagatingEffect<O>;

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
/// A `PropagatingEffect`.
pub type ContextualCausalFn<I, O, S, C> =
    fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>;
