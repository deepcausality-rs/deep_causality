use crate::{
    Causable, CausableGraph, CausableGraphExplaining, CausableGraphReasoning, CausalityError,
    CausaloidGraph, Evidence, PropagatingEffect,
};
use std::fmt::Display;
use ultragraph::GraphTraversal;

// This implementation allows an entire CausaloidGraph to be treated as a single,
// evaluatable unit. It acts as a high-level facade that orchestrates the powerful,
// centralized algorithms provided by the `CausableGraphReasoning` and
// `CausableGraphExplaining` extension traits, perfectly integrating into the
// library's architecture.
impl<T> Causable for CausaloidGraph<T>
where
    T: Clone + Display + Causable + PartialEq,
{
    /// Evaluates the entire causal graph by reasoning from its root node.
    ///
    /// This method delegates to the `evaluate_subgraph_from_cause` method from the
    /// `CausableGraphReasoning` trait to perform the actual traversal and evaluation.
    /// The final propagated effect is determined by the graph's overall active state
    /// after the evaluation is complete.
    fn evaluate(&self, evidence: &Evidence) -> Result<PropagatingEffect, CausalityError> {
        // Since the graph is guaranteed to have a single root, start evaluation there.
        let root_index = self
            .get_root_index()
            .ok_or_else(|| CausalityError("Cannot evaluate graph: Root node not found.".into()))?;

        // Delegate to the reasoning algorithm from the `CausableGraphReasoning` trait.
        // This will traverse and evaluate the entire graph from the root.
        let effect = self.evaluate_subgraph_from_cause(root_index, evidence)?;

        if matches!(effect, PropagatingEffect::Halting) {
            return Ok(PropagatingEffect::Halting);
        }

        // After evaluation, check if any sink node is active to determine the graph's summary effect.
        let is_active = self.is_active()?;
        Ok(PropagatingEffect::Deterministic(is_active))
    }

    /// Generates a human-readable explanation for the entire graph.
    ///
    /// This method delegates to the `explain_all_causes` method from the
    /// `CausableGraphExplaining` extension trait.
    fn explain(&self) -> Result<String, CausalityError> {
        // Delegate to the explaining algorithm from the `CausableGraphExplaining` trait.
        self.explain_all_causes()
    }

    /// Checks if the graph is considered "active".
    ///
    /// A graph is active if at least one of its "sink" nodes (nodes with no outgoing
    /// links) is active. This signifies that at least one full causal chain has completed
    fn is_active(&self) -> Result<bool, CausalityError> {
        // High-performance traversal requires the graph to be frozen.
        if !self.is_frozen() {
            return Err(CausalityError(
                "Graph must be frozen to check active state. Call freeze() first.".into(),
            ));
        }

        let num_nodes = self.number_nodes();
        if num_nodes == 0 {
            return Ok(false);
        }

        // To find sink nodes, we must iterate and check the out-degree of each node
        // using the available `outbound_edges` API.
        for i in 0..num_nodes {
            // A node is a sink if its iterator of outbound edges is empty.
            if self.get_graph().outbound_edges(i)?.next().is_none() {
                // Found a sink node, now check if it's active.
                let node = self.get_causaloid(i).ok_or_else(|| {
                    CausalityError(format!("is_active: Failed to get sink node at index {i}"))
                })?;

                if node.is_active()? {
                    // Short-circuit and return true on the first active sink.
                    return Ok(true);
                }
            }
        }

        // If we've iterated through all nodes and found no active sinks,
        // the graph as a whole is not active.
        Ok(false)
    }

    /// A graph is a composite type, not a singleton.
    fn is_singleton(&self) -> bool {
        false
    }
}
