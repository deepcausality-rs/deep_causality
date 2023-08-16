// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::Debug;

use ultragraph::prelude::*;

use crate::prelude::*;

#[derive(Clone)]
pub struct CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
{
    graph: CausalGraph<T>,
}

impl<T> CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
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

impl<T> Default for CausaloidGraph<T>
    where
        T: Debug + Causable + Clone + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

// See default implementation in protocols/causable_graph/graph_explaining
impl<T> CausableGraphExplaining<T> for CausaloidGraph<T> where T: Causable + Clone + PartialEq {}


// See default implementation in protocols/causable_graph/graph_explaining
impl<T> CausableGraphReasoning<T> for CausaloidGraph<T> where T: Causable + Clone + PartialEq {}

impl<T> CausableGraph<T> for CausaloidGraph<T>
    where
        T: Causable + Clone + PartialEq,
{
    fn get_graph(&self) -> &CausalGraph<T>
    {
        &self.graph
    }

    fn add_root_causaloid(&mut self, value: T) -> usize
    {
        self.graph.add_root_causaloid(value)
    }

    fn contains_root_causaloid(&self) -> bool
    {
        self.graph.contains_root_node()
    }

    fn get_root_causaloid(&self) -> Option<&T>
    {
        self.graph.get_root_causaloid()
    }

    fn get_root_index(&self) -> Option<usize>
    {
        self.graph.get_root_index()
    }

    fn get_last_index(&self) -> Result<usize, CausalityGraphError>
    {
        if !self.is_empty() {
            Ok(self.graph.get_last_index())
        } else {
            Err(CausalityGraphError("Graph is empty".to_string()))
        }
    }

    fn add_causaloid(&mut self, value: T) -> usize
    {
        self.graph.add_node(value)
    }

    fn contains_causaloid(&self, index: usize) -> bool
    {
        self.graph.contains_node(index)
    }

    fn get_causaloid(&self, index: usize) -> Option<&T>
    {
        self.graph.get_node(index)
    }

    fn remove_causaloid(&mut self, index: usize) -> Result<(), CausalGraphIndexError>
    {
        if !self.contains_causaloid(index) {
            return Err(CausalGraphIndexError(format!("index not found: {}", index)));
        };

        self.graph.remove_node(index);

        Ok(())
    }

    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> Result<(), CausalGraphIndexError>
    {
        return match self.graph.add_edge(a, b) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e)),
        }
    }

    fn add_edg_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    )
        -> Result<(), CausalGraphIndexError>
    {
        return match self.graph.add_edge_with_weight(a, b, weight) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e)),
        }
    }

    fn contains_edge(
        &self,
        a: usize,
        b: usize,
    )
        -> bool
    {
        self.graph.contains_edge(a, b)
    }

    fn remove_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> Result<(), CausalGraphIndexError>
    {
        return match self.graph.remove_edge(a, b) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e)),
        }
    }

    fn all_active(&self) -> bool
    {
        for (_, cause) in self.causes_map.iter() {
            if !cause.is_active() {
                return false;
            }
        }

        true
    }

    fn number_active(&self) -> NumericalValue
    {
        self.causes_map.iter().filter(|(_, c)| c.is_active()).count() as NumericalValue
    }

    fn percent_active(&self) -> NumericalValue
    {
        (self.number_active() / self.size() as NumericalValue) * (100 as NumericalValue)
    }

    fn size(&self) -> usize
    {
        self.graph.size()
    }

    fn is_empty(&self) -> bool
    {
        self.graph.is_empty()
    }

    fn clear(&mut self)
    {
        self.graph.clear();
    }

    fn number_edges(&self) -> usize
    {
        self.graph.number_edges()
    }

    fn number_nodes(&self) -> usize
    {
        self.graph.number_nodes()
    }
}
