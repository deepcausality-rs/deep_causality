/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Node, Sample, UncertainError};
use deep_causality_ast::ConstTree;
use deep_causality_rand::RandScalar;

/// A trait for sampling strategies.
pub trait Sampler<R: RandScalar> {
    /// Generates a single sample from the computation graph at the given sample index.
    ///
    /// `sample_index` selects the draw: the `SequentialSampler` ignores it (it draws from a
    /// stateful RNG and the index only tags the cache entry), whereas the `QmcSampler` uses it
    /// as the index of the low-discrepancy point.
    ///
    /// The result is a [`Sample<R>`] rather than the carrier's own type, because one graph serves
    /// both carriers: which of the two kinds a root produces is a fact about the root, and the
    /// carrier reading it is the one that turns a mismatch into an error.
    fn sample(
        &self,
        root_node: &ConstTree<Node<R>>,
        sample_index: u64,
    ) -> Result<Sample<R>, UncertainError>;
}
