// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use ultragraph::prelude::*;

use crate::errors::{CausalGraphIndexError, CausalityGraphError};
use crate::prelude::{Causable, NumericalValue};
use crate::protocols::causable_graph::CausalGraph;

pub trait CausableGraph<T>
    where
        T: Causable + PartialEq,
{
    // The get_graph method enables the default implementation of the
    // CausableGraphExplaining and CausableGraphReasoning traits.
    fn get_graph(&self) -> &CausalGraph<T>;
    // Root Node
    fn add_root_causaloid(&mut self, value: T) -> usize;
    fn contains_root_causaloid(&self) -> bool;
    fn get_root_causaloid(&self) -> Option<&T>;
    fn get_root_index(&self) -> Option<usize>;
    fn get_last_index(&self) -> Result<usize, CausalityGraphError>;

    // Nodes
    fn add_causaloid(&mut self, value: T) -> usize;
    fn contains_causaloid(&self, index: usize) -> bool;
    fn get_causaloid(&self, index: usize) -> Option<&T>;
    fn remove_causaloid(&mut self, index: usize) -> Result<(), CausalGraphIndexError>;

    // Edges
    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError>;
    fn add_edg_with_weight(&mut self, a: usize, b: usize, weight: u64) -> Result<(), CausalGraphIndexError>;
    fn contains_edge(&self, a: usize, b: usize) -> bool;
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError>;

    // Utils
    fn all_active(&self) -> bool;
    fn number_active(&self) -> NumericalValue;
    fn percent_active(&self) -> NumericalValue;
    fn size(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
    fn number_edges(&self) -> usize;
    fn number_nodes(&self) -> usize;

    /// Default implementation for shortest path algorithm
    fn get_shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    )
        -> Result<Vec<usize>, CausalityGraphError>
    {
        return match self.get_graph().shortest_path(start_index, stop_index) {
            Some(path) => Ok(path),
            None => Err(CausalityGraphError("No path found".to_string())),
        };
    }
}

