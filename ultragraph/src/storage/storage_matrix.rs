// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::collections::HashMap;

use petgraph::Directed;
use petgraph::graph::NodeIndex as GraphNodeIndex;
use petgraph::matrix_graph::MatrixGraph;

use crate::errors::UltraGraphError;
use crate::protocols::graph_like::GraphLike;
use crate::protocols::graph_root::GraphRoot;
use crate::protocols::graph_storage::GraphStorage;

type DefaultIx = u32;
type NodeIndex<Ix = DefaultIx> = GraphNodeIndex<Ix>;

// Edge weights need to be numerical (u64) to make shortest path algo work.
// Also, u32 is used as node node index type to bypass the fairly ancient 65k node limit
// coming from the u16 default node index default type in petgraph.
// u32 has a limit of 2^31 - 1 (4,294,967,295). NodeIndex can be at most u32 because petgraph has no implementation
// for u64 or u128. See: https://docs.rs/petgraph/latest/petgraph/graph/trait.IndexType.html
type HyperGraph<T> = MatrixGraph<T, u64, Directed, Option<u64>, u32>;

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
type IndexMap = HashMap<usize, NodeIndex>;

pub struct StorageMatrixGraph<T>
    where
        T: Copy + Clone,
{
    root_index: Option<NodeIndex>,
    graph: HyperGraph<T>,
    node_map: HashMap<NodeIndex, T>,
    index_map: IndexMap,
}

impl<T> StorageMatrixGraph<T>
    where
        T: Copy + Clone,
{
    pub fn new() -> Self {
        Self {
            root_index: None,
            graph: MatrixGraph::default(),
            node_map: HashMap::new(),
            index_map: HashMap::new(),
        }
    }

    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            root_index: None,
            graph: MatrixGraph::with_capacity(capacity),
            node_map: HashMap::with_capacity(capacity),
            index_map: HashMap::with_capacity(capacity),
        }
    }
}


impl<T> Default for StorageMatrixGraph<T>
    where
        T: Copy + Clone
{
    fn default() -> Self {
        Self::new()
    }
}


impl<T> GraphStorage<T> for StorageMatrixGraph<T>
    where
        T: Copy
{
    fn size(&self) -> usize {
        self.graph.node_count()
    }

    fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    fn number_nodes(&self) -> usize {
        self.graph.node_count()
    }

    fn number_edges(&self) -> usize {
        self.graph.edge_count()
    }

    fn clear(&mut self) {
        self.graph.clear();
        self.node_map.clear();
        self.index_map.clear();
        self.root_index = None;
    }
}


impl<T> GraphRoot<T> for StorageMatrixGraph<T>
    where
        T: Copy + Clone,
{
    fn add_root_node(&mut self, value: T) -> usize
    {
        let idx = self.add_node(value);
        let root_index = NodeIndex::new(idx);
        self.root_index = Some(root_index);
        self.index_map.insert(root_index.index(), root_index);
        root_index.index()
    }

    fn contains_root_node(&self) -> bool
    {
        self.root_index.is_some()
    }

    fn get_root_node(&self) -> Option<&T>
    {
        if self.contains_root_node()
        {
            self.node_map.get(&self.root_index.unwrap())
        } else {
            None
        }
    }

    fn get_root_index(&self) -> Option<usize>
    {
        if self.contains_root_node() {
            Some(self.root_index.unwrap().index())
        } else {
            None
        }
    }

    fn get_last_index(&self) -> Result<usize, UltraGraphError>
    {
        if !self.is_empty() {
            Ok(self.node_map.len() - 1)
        } else {
            Err(UltraGraphError("Graph is empty".to_string()))
        }
    }
}


impl<T> GraphLike<T> for StorageMatrixGraph<T>
    where
        T: Copy + Clone,
{
    fn add_node(&mut self, value: T) -> usize
    {
        let node_index = self.graph.add_node(value);
        self.node_map.insert(node_index, value);
        self.index_map.insert(node_index.index(), node_index);
        node_index.index()
    }

    fn contains_node(&self, index: usize) -> bool
    {
        self.index_map.get(&index).is_some()
    }

    fn get_node(&self, index: usize) -> Option<&T>
    {
        if !self.contains_node(index) {
            None
        } else {
            let k = self.index_map.get(&index).expect("index not found");
            self.node_map.get(k)
        }
    }

    fn remove_node(&mut self, index: usize) -> Result<(), UltraGraphError> {
        if !self.contains_node(index) {
            return Err(UltraGraphError(format!("index {} not found", index)));
        };

        let k = self.index_map.get(&index).unwrap();
        self.graph.remove_node(*k);
        self.node_map.remove(k);
        self.index_map.remove(&k.index());
        Ok(())
    }

    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError> {
        if !self.contains_node(a) {
            return Err(UltraGraphError(format!("index a {} not found", a)));
        };

        if !self.contains_node(b) {
            return Err(UltraGraphError(format!("index b {} not found", b)));
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");
        self.graph.add_edge(*k, *l, 0);
        Ok(())
    }

    fn add_edge_with_weight(&mut self, a: usize, b: usize, weight: u64) -> Result<(), UltraGraphError> {
        if !self.contains_node(a) {
            return Err(UltraGraphError(format!("index a {} not found", a)));
        };

        if !self.contains_node(b) {
            return Err(UltraGraphError(format!("index b {} not found", b)));
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");
        self.graph.add_edge(*k, *l, weight);
        Ok(())
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool
    {
        if !self.contains_node(a) || !self.contains_node(b) {
            return false;
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");
        self.graph.has_edge(*k, *l)
    }

    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError> {
        if !self.contains_node(a) {
            return Err(UltraGraphError("index a not found".into()));
        };

        if !self.contains_node(b) {
            return Err(UltraGraphError("index b not found".into()));
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.remove_edge(*k, *l);

        Ok(())
    }
}
