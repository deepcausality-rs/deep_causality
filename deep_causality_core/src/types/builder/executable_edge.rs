/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::fmt::{Display, Formatter};

/// A simple directed edge stored as an adjacency list.
#[derive(Clone, Debug)]
pub struct ExecutableEdge {
    pub(crate) from: usize,
    pub(crate) to: usize,
}

impl ExecutableEdge {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}

impl ExecutableEdge {
    pub fn from(&self) -> usize {
        self.from
    }

    pub fn to(&self) -> usize {
        self.to
    }
}

impl Display for ExecutableEdge {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "ExecutableEdge(from: {}, to: {})", self.from, self.to)
    }
}
