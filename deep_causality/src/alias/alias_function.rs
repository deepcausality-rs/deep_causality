/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AssumptionError, ParentEffects};
use deep_causality_core::{CausalEffect, PropagatingEffect, PropagatingProcess};

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
/// It evaluates the incoming `CausalEffect<I>` against its own static configuration and the shared
/// context to produce a `PropagatingProcess<O, S, C>`.
///
/// # Arguments
///
/// * `effect` - The `CausalEffect<I>` flowing through the graph during reasoning.
/// * `state`  - The state `S` carried by the incoming process (preserved across evaluation).
/// * `context` - The optional `Option<C>` context carried by the incoming process.
///
/// # Returns
///
/// A `PropagatingProcess<O, S, C>` whose `state` and `context` are returned to the caller.
pub type ContextualCausalFn<I, O, S, C> =
    fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>;

/// The unified function signature for singleton causaloids authored on the
/// stateful evaluation path.
///
/// Structurally identical to [`ContextualCausalFn`] (both resolve to the same
/// `fn` pointer type), this alias is a clearly-named ergonomic marker at the
/// closure-author site. Use it when authoring closures intended to be evaluated
/// via [`crate::StatefulMonadicCausable::evaluate_stateful`] or one of the
/// stateful collection / graph reasoning methods.
///
/// The existing [`crate::Causaloid::new_with_context`] constructor accepts this
/// alias as-is — no separate "stateful" constructor exists. Statefulness on a
/// `Causaloid` is a property of the **evaluation call** (which trait method is
/// invoked), not of the constructor.
///
/// # Arguments
///
/// * `effect` - A reference to the `CausalEffect` flowing through the graph during reasoning.
/// * `state`  - The state carried by the incoming process (preserved across evaluation).
/// * `context` - The optional context carried by the incoming process.
///
/// # Returns
///
/// A `PropagatingProcess<O, S, C>` whose `state` and `context` are returned to
/// the caller intact (no defaulting, no discarding).
pub type StatefulContextualCausalFn<I, O, S, C> =
    fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>;

/// The join mechanism at a reconvergence node: reduces the labeled effects of the
/// parents that fired to the single effect the node consumes as input.
///
/// At a fan-in the reasoning engine hands the node its fired parents keyed by parent
/// node index (a [`ParentEffects`]); this function collapses them to one
/// `PropagatingEffect<I>` — the node's incoming effect — after which the node is
/// evaluated normally by its `causal_fn` / `context_causal_fn`. Because the parents
/// are keyed, any function is admissible (asymmetric mechanisms are the norm), and
/// determinism comes from the keying, not from any algebraic law on the mechanism.
///
/// A join of one fired parent is the identity (the engine passes that parent's effect
/// through unchanged and never invokes this function), so linear/tree graphs are
/// unaffected.
///
/// # Arguments
///
/// * `parents` - The fired parent effects, keyed by parent node index.
///
/// # Returns
///
/// The single `PropagatingEffect<I>` fed to the node as its incoming effect.
pub type JoinFn<I> = fn(&ParentEffects<I>) -> PropagatingEffect<I>;

/// A join mechanism that additionally reads static per-node configuration carried on
/// the causaloid's context channel (mirroring the [`ContextualCausalFn`] config-on-
/// context pattern).
///
/// The kernel joins (e.g. a linear structural-equation combine) need weights/coefficients
/// that a bare `fn` pointer cannot capture; those ride the causaloid's `context: Option<CTX>`
/// field and are passed to this function as `Option<&CTX>`. `None` means no context was
/// configured — a kernel treats that as a configuration error.
///
/// # Arguments
///
/// * `parents` - The fired parent effects, keyed by parent node index.
/// * `context` - The causaloid's configuration carried on its context channel.
///
/// # Returns
///
/// The single `PropagatingEffect<I>` fed to the node as its incoming effect.
pub type ContextualJoinFn<I, CTX> = fn(&ParentEffects<I>, Option<&CTX>) -> PropagatingEffect<I>;
