/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// One iterator type over the two states' traversal iterators, so the container's traversal
/// returns a single opaque type without a heap allocation.
pub(crate) enum TraversalIter<F, D> {
    Frozen(F),
    Dynamic(D),
}

impl<F, D> Iterator for TraversalIter<F, D>
where
    F: Iterator<Item = usize>,
    D: Iterator<Item = usize>,
{
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        match self {
            Self::Frozen(iter) => iter.next(),
            Self::Dynamic(iter) => iter.next(),
        }
    }
}
