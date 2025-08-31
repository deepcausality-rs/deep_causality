/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ComputationNode;
use ultragraph::UltraGraphContainer;

// This is now a non-generic type alias.
// The graph contains nodes of different logical types.
pub type UncertainGraph = UltraGraphContainer<ComputationNode, ()>;
