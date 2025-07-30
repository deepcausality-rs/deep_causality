use crate::{
    Causable, CausableGraph, CausableGraphExplaining, CausableGraphReasoning, CausalityError,
    CausaloidGraph, PropagatingEffect,
};
use std::fmt::Display;

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
    fn evaluate(&self, effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        // Since the graph is guaranteed to have a single root, start evaluation there.
        let root_index = self
            .get_root_index()
            .ok_or_else(|| CausalityError("Cannot evaluate graph: Root node not found.".into()))?;

        // Delegate to the reasoning algorithm from the `CausableGraphReasoning` trait.
        // This will traverse and evaluate the entire graph from the root.
        let effect = self.evaluate_subgraph_from_cause(root_index, effect)?;

        if matches!(effect, PropagatingEffect::Halting) {
            return Ok(PropagatingEffect::Halting);
        }

        Ok(effect)
    }

    /// Generates a human-readable explanation for the entire graph.
    ///
    /// This method delegates to the `explain_all_causes` method from the
    /// `CausableGraphExplaining` extension trait.
    fn explain(&self) -> Result<String, CausalityError> {
        // Delegate to the explaining algorithm from the `CausableGraphExplaining` trait.
        self.explain_all_causes()
    }

    /// A graph is a composite type, not a singleton.
    fn is_singleton(&self) -> bool {
        false
    }
}
