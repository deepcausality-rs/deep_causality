/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The linear structural-equation fan-in kernel.
//!
//! `linear_join` computes `v_n = bias + Σ_{p ∈ fired} weights[p] · v_p` over the labeled
//! parent effects at a reconvergence node — the classical **linear structural causal
//! model** mechanism (Pearl, *Causality*, 2nd ed., CUP 2009, §1.4, linear SEMs). It is
//! written as a [`ContextualJoinFn`](crate::ContextualJoinFn): the weights and bias ride
//! the node's context channel as a [`LinearJoin`] so a bare `fn` pointer stays capture-free.
//!
//! The kernel is generic over [`Scalar`], so precision is a free parameter (`f32` / `f64` /
//! `Float106`) and a dual number passes through unchanged: seeding a parent as
//! `Dual::variable` makes the `ε` part of the output the sensitivity `∂v_n/∂v_p = weights[p]`.
//! Multilinearity in the labeled parents is exactly what makes `LinearJoin` the diagonal
//! classical shadow of a quantum causal-model channel factor `ρ_{A|Pa(A)}` (Lorenz,
//! *Quantum causal models*, Synthese 200:424, 2022), so the same kernel shape lifts to the
//! complex / operator-valued tiers without redefinition.

use crate::{CausalityError, CausalityErrorEnum, ParentEffects, PropagatingEffect};
use deep_causality_num::Scalar;
use std::collections::BTreeMap;
use std::fmt::Debug;

/// Configuration for [`linear_join`]: per-parent weights (keyed by parent node index)
/// and an additive bias.
///
/// Carried on a reconvergence node's context channel via
/// [`Causaloid::new_with_context_join`](crate::Causaloid::new_with_context_join).
#[derive(Clone)]
pub struct LinearJoin<R: Scalar> {
    /// Coefficient applied to each labeled parent's value. A fired parent with no entry
    /// contributes nothing (coefficient zero).
    weights: BTreeMap<usize, R>,
    /// The additive bias / intercept term.
    bias: R,
}

impl<R: Scalar> LinearJoin<R> {
    /// Builds a linear-join configuration from per-parent weights and a bias.
    #[inline]
    pub fn new(weights: BTreeMap<usize, R>, bias: R) -> Self {
        LinearJoin { weights, bias }
    }

    /// The per-parent weight map (keyed by parent node index).
    #[inline]
    pub fn weights(&self) -> &BTreeMap<usize, R> {
        &self.weights
    }

    /// The additive bias term.
    #[inline]
    pub fn bias(&self) -> R {
        self.bias
    }
}

/// The linear structural-equation join: `bias + Σ_{p ∈ fired} weights[p] · v_p`.
///
/// A [`ContextualJoinFn`](crate::ContextualJoinFn) over `LinearJoin<R>`. Parents are
/// summed in ascending node-index order (the [`ParentEffects`] iteration order), so the
/// result is independent of the order in which parents fired. Per-parent policy:
///
/// - a parent carrying a command (`RelayTo`) on its value channel yields a `CausalityError`;
/// - a parent whose value is `None` (`Pure(None)`) contributes nothing;
/// - a fired parent with no weight entry contributes nothing (coefficient zero);
/// - a missing configuration (`config = None`) yields a `CausalityError`.
///
/// Parent *errors* are short-circuited by the engine before the join runs, so they are
/// not handled here.
pub fn linear_join<R: Scalar + Default + Debug>(
    parents: &ParentEffects<R>,
    config: Option<&LinearJoin<R>>,
) -> PropagatingEffect<R> {
    let config = match config {
        Some(c) => c,
        None => {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                "linear_join: missing LinearJoin configuration on the node context".into(),
            )));
        }
    };

    let mut acc = config.bias;
    for (parent_index, effect) in parents.iter() {
        if effect.command_target().is_some() {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                format!(
                    "linear_join: parent {parent_index} carries a command (RelayTo) on its value channel"
                ),
            )));
        }
        // A `Pure(None)` value and a fired parent with no weight both contribute nothing.
        if let (Some(weight), Some(value)) = (config.weights.get(&parent_index), effect.value()) {
            acc = acc + (*weight * *value);
        }
    }

    PropagatingEffect::from_value(acc)
}
