// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::Display;
use ultragraph::prelude::*;

use crate::errors::{CausalGraphIndexError, CausalityGraphError};
use crate::prelude::{
    Causable, CausableGraph, CausableGraphExplaining, CausableGraphReasoning, CausalGraph,
    NumericalValue,
};

mod causable_graph;
mod default;

#[derive(Clone)]
pub struct CausaloidGraph<T>
where
    T: Causable + PartialEq + Clone + Display,
{
    graph: CausalGraph<T>,
}

impl<T> CausaloidGraph<T>
where
    T: Causable + PartialEq + Clone + Display,
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
