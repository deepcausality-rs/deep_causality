// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use petgraph::Directed;
use petgraph::graph::NodeIndex as GraphNodeIndex;
use petgraph::matrix_graph::MatrixGraph;

type DefaultIx = u32;
type NodeIndex<Ix = DefaultIx> = GraphNodeIndex<Ix>;

// Edge weights need to be numerical (u64) to make shortest path algo work.
// Also, u32 is used as node node index type to bypass the fairly ancient 65k node limit
// coming from the u16 default node index default type in petgraph.
// u32 has a limit of 2^31 - 1 (4,294,967,295). NodeIndex can be at most u32 because petgraph has no implementation
// for u64 or u128. See: https://docs.rs/petgraph/latest/petgraph/graph/trait.IndexType.html
pub type HyperGraph<T> = MatrixGraph<T, u64, Directed, Option<u64>, u32>;


//
// Petgraph has no good way to retrieve a specific node hence the hashmap as support structure
// for the get & contains node methods. Given that the context will be embedded as a reference
// into many causaloids, it is safe to say that nodes from the context will be retrieved quite
// freequently therefore the direct access from the hashmap should speed things up.
//
// Ideally, the hashmap should hold only a reference to the contextoid in the graph,
// but this causes trouble with the borrow checker hence the node is stored as a value.
// As a consequence, all nodes stores in the graph and hashmap must implement the clone trait.
//
// While this is inefficient and memory intensive for large context graphs, it should be fine
// for small to medium graphs.
// type CtxMap<'l, CNT,D, S, T, ST> = HashMap<NodeIndex, CNT<D, S, T, ST>>;
// //
//
// type IndexMap = HashMap<usize, NodeIndex>;