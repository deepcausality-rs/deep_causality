/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Causaloid, PropagatingEffect};
use crate::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
mod display;
mod eval;
mod getter;

/// A `CausalState` represents a state in a causal state machine (CSM) that can be evaluated
/// based on causal conditions.
///
/// In a CSM, states are paired with actions. When a state's conditions are met (evaluated to true),
/// the associated action is triggered.
///
/// # Purpose
/// `CausalState` encapsulates a reference to a causaloid that defines when the state should be
/// considered active, along with:
/// - An identifier for the state
/// - A version number for tracking changes
/// - Data that can be used for evaluation
///
/// # Usage
/// `CausalState` is typically used in conjunction with `CausalAction` in a state-action pair
/// within a causal state machine (CSM). The CSM evaluates states and, when conditions are met,
/// fires the associated actions.
///
#[allow(clippy::type_complexity)]
#[derive(Clone, Debug)]
pub struct CausalState<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Unique identifier for the state
    id: usize,
    /// Version number for tracking changes to the state
    version: usize,
    /// Numerical data used for state evaluation
    data: PropagatingEffect,
    /// Reference to a causaloid that defines when this state is active
    causaloid: Causaloid<D, S, T, ST, SYM, VS, VT>,
}

impl<D, S, T, ST, SYM, VS, VT> CausalState<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    pub fn new(
        id: usize,
        version: usize,
        data: PropagatingEffect,
        causaloid: Causaloid<D, S, T, ST, SYM, VS, VT>,
    ) -> Self {
        Self {
            id,
            version,
            data,
            causaloid,
        }
    }
}
