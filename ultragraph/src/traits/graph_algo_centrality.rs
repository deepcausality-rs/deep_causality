/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{GraphError, GraphView};

pub trait CentralityGraphAlgorithms<N, W>: GraphView<N, W> {
    /// Calculates the betweenness centrality of each node in the graph.
    ///
    /// Betweenness centrality measures a node's importance by counting how often it
    /// appears on the shortest paths between all other pairs of nodes. It is a powerful
    /// metric for identifying bottlenecks, bridges, and critical control points in a network.
    ///
    /// The implementation uses Brandes' algorithm, which is highly efficient.
    ///
    /// # Arguments
    ///
    /// * `directed`: If `true`, the calculation considers the graph's edge directions.
    ///   If `false`, all edges are treated as bidirectional.
    /// * `normalized`: If `true`, the centrality scores are normalized by dividing by the
    ///   number of possible pairs of nodes. This allows for comparison between graphs of
    ///   different sizes. If `false`, the raw count of paths is returned.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `(node_index, centrality_score)` tuples.
    /// The centrality score is an `f64` representing the node's importance.
    /// The vector is unsorted.
    fn betweenness_centrality(
        &self,
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError>;

    /// Calculates betweenness centrality across a specific set of critical pathways.
    ///
    /// This function is a highly efficient tool for targeted analysis, such as root cause
    /// investigation or identifying bottlenecks in specific set of graph pathways. Instead of
    /// analyzing all possible paths, it only considers the shortest paths between the
    /// start and end nodes of the pathways you provide.
    ///
    /// The algorithm correctly routes these paths through the entire graph, ensuring that
    /// the results are accurate even if the shortest path temporarily leaves a conceptual
    /// "subgraph".
    ///
    /// # Arguments
    /// * `pathways`: A slice of `(start_node, end_node)` tuples defining the pathways to analyze.
    /// * `directed`: If `true`, considers edge directions.
    /// * `normalized`: If `true`, normalizes scores by the number of provided pathways.
    ///
    /// # Returns
    /// An unsorted vector of `(node_index, centrality_score)` tuples for nodes that lie on
    /// one or more of the specified pathways.
    ///
    fn pathway_betweenness_centrality(
        &self,
        pathways: &[(usize, usize)],
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError>;
}
