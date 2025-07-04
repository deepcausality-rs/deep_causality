/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![forbid(unsafe_code)]

use crate::{CsmGraph, DynamicGraph};

// This is the core of the refactor plan...
pub enum GraphState<N, W>
where
    W: Default,
{
    Dynamic(DynamicGraph<N, W>), // The "unfrozen" state, for mutation
    Static(CsmGraph<N, W>),      // The "frozen" state, for analysis
}
