// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use super::*;

// See default implementation in protocols/causaloid_graph/graph_explaining. Requires CausableGraph impl.
impl<T> CausableGraphExplaining<T> for CausaloidGraph<T> where T: Causable + PartialEq {}

// See default implementation in protocols/causaloid_graph/graph_explaining. Requires CausableGraph impl.
impl<T> CausableGraphReasoning<T> for CausaloidGraph<T> where T: Causable + PartialEq {}

impl<T> CausableGraph<T> for CausaloidGraph<T>
where
    T: Causable + PartialEq,
{
    fn get_graph(&self) -> &CausalGraph<T> {
        &self.graph
    }

    fn add_root_causaloid(&mut self, value: T) -> usize {
        self.graph.add_root_node(value)
    }

    fn contains_root_causaloid(&self) -> bool {
        self.graph.contains_root_node()
    }

    fn get_root_causaloid(&self) -> Option<&T> {
        self.graph.get_root_node()
    }

    fn get_root_index(&self) -> Option<usize> {
        self.graph.get_root_index()
    }

    fn get_last_index(&self) -> Result<usize, CausalityGraphError> {
        if !self.is_empty() {
            let last_index = self
                .graph
                .get_last_index()
                .expect("Could not get last index");

            Ok(last_index)
        } else {
            Err(CausalityGraphError("Graph is empty".to_string()))
        }
    }

    fn add_causaloid(&mut self, value: T) -> usize {
        self.graph.add_node(value)
    }

    fn contains_causaloid(&self, index: usize) -> bool {
        self.graph.contains_node(index)
    }

    fn get_causaloid(&self, index: usize) -> Option<&T> {
        self.graph.get_node(index)
    }

    fn remove_causaloid(&mut self, index: usize) -> Result<(), CausalGraphIndexError> {
        match self.graph.remove_node(index) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError> {
        match self.graph.add_edge(a, b) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    fn add_edg_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    ) -> Result<(), CausalGraphIndexError> {
        match self.graph.add_edge_with_weight(a, b, weight) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.graph.contains_edge(a, b)
    }

    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError> {
        match self.graph.remove_edge(a, b) {
            Ok(_) => Ok(()),
            Err(e) => Err(CausalGraphIndexError(e.to_string())),
        }
    }

    fn all_active(&self) -> bool {
        for cause in self.graph.get_all_nodes() {
            if !cause.is_active() {
                return false;
            }
        }

        true
    }

    fn number_active(&self) -> NumericalValue {
        self.graph
            .get_all_nodes()
            .iter()
            .filter(|c| c.is_active())
            .count() as NumericalValue
    }

    fn percent_active(&self) -> NumericalValue {
        (self.number_active() / self.size() as NumericalValue) * (100 as NumericalValue)
    }

    fn size(&self) -> usize {
        self.graph.size()
    }

    fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }

    fn clear(&mut self) {
        self.graph.clear();
    }

    fn number_edges(&self) -> usize {
        self.graph.number_edges()
    }

    fn number_nodes(&self) -> usize {
        self.graph.number_nodes()
    }
}
