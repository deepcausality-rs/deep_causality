
// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use ultragraph::prelude::*;

use crate::errors::{CausalGraphIndexError, CausalityGraphError};
use crate::prelude::{Causable, CausableGraph, CausableGraphExplaining, CausableGraphReasoning, CausalGraph, NumericalValue};

mod default;
mod causable_graph;

#[derive(Clone)]
pub struct CausaloidGraph<T>
    where
        T: Causable + PartialEq,
{
    graph: CausalGraph<T>,
}

impl<T> CausaloidGraph<T>
    where
        T: Causable + PartialEq,
{
    pub fn new() -> Self {
        Self {
            graph: ultragraph::new_with_matrix_storage(500),
        }
    }

    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            graph: ultragraph::new_with_matrix_storage(capacity),
        }
    }
}
