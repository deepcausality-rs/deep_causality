/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Building a `QmcSampler` from a graph root the public constructors cannot produce.

use crate::{Node, QmcSampler, UncertainError, UncertainScalar};
use deep_causality_ast::ConstTree;

/// Runs the QMC static-structure pre-pass over a hand-built graph.
///
/// [`QmcSampler::new`] and [`QmcSampler::for_bool`] take a carrier, and no carrier method builds a
/// `BindOp` — the drawn structure would depend on a sampled value, which is exactly what QMC
/// cannot have. The guard against it is therefore unreachable from the public API and would go
/// untested without this.
pub fn qmc_sampler_from_root<R: UncertainScalar>(
    root: &ConstTree<Node<R>>,
    seed: Option<u64>,
) -> Result<QmcSampler, UncertainError> {
    QmcSampler::from_root_node(root, seed)
}
