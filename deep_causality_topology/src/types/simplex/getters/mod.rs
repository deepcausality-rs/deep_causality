/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Simplex;

impl Simplex {
    /// The simplex's vertex indices, **strictly increasing and without repeats**.
    ///
    /// Every construction path preserves this: [`Simplex::new`] sorts its input,
    /// and internally generated faces are built from an already-sorted parent.
    ///
    /// This ordering is not incidental. It is the *branching structure* the cup
    /// product is defined against: the Alexander–Whitney split of a `(p+q)`-simplex
    /// pairs its leading `p+1` vertices with its trailing `q+1` vertices, and
    /// "leading" and "trailing" are meaningless without a total order on the
    /// vertices that agrees across every simplex sharing them.
    ///
    /// Chen & Tata (arXiv:2106.05274) §II state the dependency directly: these
    /// constructions "require a branching structure on the triangulation in order
    /// to determine local vertex orderings, whereas the boundary operators did
    /// not." Callers building faces by slicing this list may rely on the order;
    /// any change to it silently changes the cup product.
    ///
    /// See [`SplittableCell::split`](crate::SplittableCell::split) for the
    /// consumer, and `deep_causality_physics::kernels::mhd` for an existing one.
    pub fn vertices(&self) -> &Vec<usize> {
        &self.vertices
    }
}
