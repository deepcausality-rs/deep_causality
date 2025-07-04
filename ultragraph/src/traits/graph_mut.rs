use crate::{GraphError, GraphView};

pub trait GraphMut<N, W>: GraphView<N, W> {
    // Node Mutation
    fn add_node(&mut self, node: N) -> usize;
    fn update_node(&mut self, index: usize, node: N) -> Result<(), GraphError>;

    /// Removes a node from the graph, effectively "tombstoning" it.
    ///
    /// This operation marks the node as removed but does not re-index the graph,
    /// preserving the stability of existing node indices. Edges connected to
    /// this node are logically removed but remain in the adjacency lists until
    /// the graph is `freeze`n.
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if the index is out of bounds or if the
    /// node at that index has already been removed.
    fn remove_node(&mut self, index: usize) -> Result<(), GraphError>;

    // Edge Mutation
    fn add_edge(&mut self, a: usize, b: usize, weight: W) -> Result<(), GraphError>;
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), GraphError>;

    // Root Node Mutation
    fn add_root_node(&mut self, node: N) -> usize;

    // Graph-wide Mutation
    fn clear(&mut self);
}
