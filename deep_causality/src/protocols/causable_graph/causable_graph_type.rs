// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use petgraph::matrix_graph::MatrixGraph;
use petgraph::Directed;
use crate::errors::{CausalGraphIndexError, CausalityGraphError};
use crate::prelude::{Causable, NumericalValue};


// Edge weights need to be numerical (u64) to make shortest path algo work.
pub type CausalGraph<T> = MatrixGraph<T, u64, Directed, Option<u64>, u32>;

pub trait CausableGraph<T>
    where
        T: Causable + PartialEq,
{
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
    fn get_graph(&self) -> &CausalGraph<T>;
}

