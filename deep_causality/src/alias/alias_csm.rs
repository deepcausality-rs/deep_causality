/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalAction, CausalState};
use std::collections::HashMap;

/// A tuple consisting of a causal state and an associated causal action.
///
/// This is used to represent the result of state-action reasoning steps.
pub type StateAction<I, O, D, S, T, ST, SYM, VS, VT> =
    (CausalState<I, O, D, S, T, ST, SYM, VS, VT>, CausalAction);

pub type CSMMap<I, O, D, S, T, ST, SYM, VS, VT> =
    HashMap<usize, StateAction<I, O, D, S, T, ST, SYM, VS, VT>>;

pub type CSMStateActions<I, O, D, S, T, ST, SYM, VS, VT> =
    [StateAction<I, O, D, S, T, ST, SYM, VS, VT>];
