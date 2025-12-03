/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::marker::PhantomData;

/// A "Ghost Handle" representing a node in the graph.
/// The `In` and `Out` types are phantom; they carry no runtime weight
/// but prevent invalid connections.
pub struct NodeType<In, Out> {
    pub(crate) id: usize,
    _marker: PhantomData<(In, Out)>,
}

impl<In, Out> NodeType<In, Out> {
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<In, Out> Clone for NodeType<In, Out> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<In, Out> Copy for NodeType<In, Out> {}

impl<In, Out> NodeType<In, Out> {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}
