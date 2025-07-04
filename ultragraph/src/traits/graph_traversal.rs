use crate::{GraphError, GraphView};

pub trait GraphTraversal<N, W>: GraphView<N, W> {
    // --- Traversal ---

    /// Returns a non-allocating iterator over the direct successors (outgoing edges) of node `a`.
    ///
    /// This method provides a direct, high-performance view into the graph's internal
    /// structure without any intermediate memory allocations.
    ///
    /// # Returns
    /// A `Result` containing an iterator that yields the `usize` indices of the neighbor nodes.
    fn outbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError>;

    /// Returns a non-allocating iterator over the direct predecessors (incoming edges) of node `a`.
    fn inbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError>;
}
