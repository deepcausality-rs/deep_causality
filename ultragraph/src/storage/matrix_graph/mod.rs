// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use ahash::AHashMap;
use petgraph::Directed;
use petgraph::graph::NodeIndex as GraphNodeIndex;
use petgraph::matrix_graph::MatrixGraph;

mod default;
mod graph_storage;
mod graph_root;
mod graph_like;

type DefaultIx = u32;
type NodeIndex<Ix = DefaultIx> = GraphNodeIndex<Ix>;

// Edge weights need to be numerical (u64) to make the shortest path algo work.
// Also, u32 is used as custom node node index type to bypass the fairly ancient 65k node limit
// coming from the u16 default node index default type in petgraph. The u16 default index limit
// was handled with a wrap-around meaning, after adding 65k nodes to the graph, the index counter
// wrapped back to 0 so that the next insert would start overwriting data in the graph....
//
// u32 has a limit of about 4 billion nodes or, more precisely, 2^31 - 1 (4,294,967,295).
// NodeIndex can be at most u32 because petgraph has no implementation for u64 or u128 index.
// See: https://docs.rs/petgraph/latest/petgraph/graph/trait.IndexType.html
//
// Graph is directed by default because otherwise neighbors would also return all incoming edges
// and that is undesirable in the current use case in causality and context graphs.
// These graphs are always directed graphs and therefore neighbors should only return outgoing edges.
type HyperGraph<T> = MatrixGraph<T, u64, Directed, Option<u64>, u32>;

// IndexMap literally maps between the usize index used in the public API and the
// stable node index used in petgraph.
type IndexMap = AHashMap<usize, NodeIndex>;

// node_map stores the actual nodes in a hashmap betcause
// petgraph does not have a good way to retrieve a specific node from the graph.
//
// Given that the context will be embedded as a reference
// into many causaloids, it is safe to say that nodes from the context will be retrieved quite
// frequently therefore the direct access from the hashmap should speed things up.
//
// For performance reasons, the AHash hashmap is used instead of the std hashmap.
// AHash is the fastest, DOS resistant hash currently available in Rust...
// https://github.com/tkaitchuck/aHash
type NodeMap<T> = AHashMap<NodeIndex, T>;

// RootIndex is a convenience accessor for the root node index.
// There are a use cases where the root node index is not at position 0,
// but most graph algorithms require the root index as a starting point regardless of its actual position.
// By default, RootIndex is set to None and must be explicitly set to a valid index by calling
// set_root_index(). If root index is not set, then get_root_index() will return None.
type RootIndex = Option<NodeIndex>;

#[derive(Clone)]
pub struct UltraMatrixGraph<T>
{
    root_index: RootIndex,
    graph: HyperGraph<bool>,
    node_map: NodeMap<T>,
    index_map: IndexMap,
}

impl<T> UltraMatrixGraph<T>
{
    pub fn new() -> Self {
        Self {
            root_index: None,
            graph: MatrixGraph::default(),
            node_map: AHashMap::new(),
            index_map: AHashMap::new(),
        }
    }

    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            root_index: None,
            graph: MatrixGraph::with_capacity(capacity),
            node_map: AHashMap::with_capacity(capacity),
            index_map: AHashMap::with_capacity(capacity),
        }
    }
}
