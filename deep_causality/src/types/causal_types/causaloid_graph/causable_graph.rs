/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use super::*;
use crate::CausalityError;

// See default implementation in protocols/causaloid_graph/graph_explaining. Requires CausableGraph impl.
impl<T> CausableGraphExplaining<T> for CausaloidGraph<T> where
    T: Clone + Display + Causable + PartialEq
{
}

// See default implementation in protocols/causaloid_graph/graph_explaining. Requires CausableGraph impl.
impl<T> CausableGraphReasoning<T> for CausaloidGraph<T> where
    T: Clone + Display + Causable + PartialEq
{
}

#[allow(clippy::type_complexity)]
impl<T> CausableGraph<T> for CausaloidGraph<T>
where
    T: Clone + Display + Causable + PartialEq,
{
    fn is_frozen(&self) -> bool {
        self.graph.is_frozen()
    }

    fn freeze(&mut self) {
        self.graph.freeze()
    }

    fn unfreeze(&mut self) {
        self.graph.unfreeze()
    }

    fn get_graph(&self) -> &CausalGraph<T> {
        &self.graph
    }

    fn add_root_causaloid(&mut self, value: T) -> Result<usize, CausalityGraphError> {
        match self.graph.add_root_node(value) {
            Ok(index) => Ok(index),
            Err(e) => Err(CausalityGraphError(e.to_string())),
        }
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
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        // Handle the Option from the underlying graph implementation with a precise error.
        self.graph.get_last_index().ok_or_else(|| {
            CausalityGraphError("Failed to get last index from a non-empty graph".to_string())
        })
    }

    fn add_causaloid(&mut self, value: T) -> Result<usize, CausalityGraphError> {
        match self.graph.add_node(value) {
            Ok(index) => Ok(index),
            Err(e) => Err(CausalityGraphError(e.to_string())),
        }
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
        match self.graph.add_edge(a, b, 0) {
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
        match self.graph.add_edge(a, b, weight) {
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

    /// Checks if all causaloids in the graph are active.
    /// Propagates any error encountered while checking a causaloid's state.
    fn all_active(&self) -> Result<bool, CausalityError> {
        for cause in self.graph.get_all_nodes() {
            // Use `?` to handle the Result from is_active()
            if !cause.is_active()? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Counts the number of active causaloids in the graph.
    /// Propagates any error encountered while checking a causaloid's state.
    fn number_active(&self) -> Result<NumericalValue, CausalityError> {
        let mut count = 0;
        for cause in self.graph.get_all_nodes() {
            // Use `?` to handle the Result from is_active()
            if cause.is_active()? {
                count += 1;
            }
        }
        Ok(count as NumericalValue)
    }

    /// Calculates the percentage of active causaloids in the graph.
    /// Propagates any error encountered during the calculation.
    fn percent_active(&self) -> Result<NumericalValue, CausalityError> {
        let total = self.size() as NumericalValue;
        if total == 0.0 {
            return Ok(0.0);
        }
        // Use `?` to handle the Result from number_active()
        let active_count = self.number_active()?;
        Ok((active_count / total) * 100.0)
    }

    fn size(&self) -> usize {
        self.graph.number_nodes()
    }

    fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }

    fn clear(&mut self) {
        let _ = self.graph.clear();
    }

    fn number_edges(&self) -> usize {
        self.graph.number_edges()
    }

    fn number_nodes(&self) -> usize {
        self.graph.number_nodes()
    }

    /// Counts the number of nodes that are known to be active, ignoring unevaluated nodes.
    ///
    /// This is a lenient check useful for inspecting partially evaluated graphs.
    /// It treats any unevaluated node as "not active" for the purpose of the count.
    fn count_known_active(&self) -> NumericalValue {
        let mut count = 0;
        for cause in self.graph.get_all_nodes() {
            // is_active() returns Err if not evaluated.
            // unwrap_or(false) treats unevaluated as inactive for the count.
            if cause.is_active().unwrap_or(false) {
                count += 1;
            }
        }
        count as NumericalValue
    }
}
