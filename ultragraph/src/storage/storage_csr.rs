// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use std::collections::HashMap;

use petgraph::csr::Csr;
use petgraph::graph::NodeIndex as GraphNodeIndex;
use petgraph::prelude::*;

use crate::prelude::{GraphLike, GraphRoot, GraphStorage, UltraGraphError};

type DefaultIx = u32;
type NodeIndex<Ix = DefaultIx> = GraphNodeIndex<Ix>;

type CsrGraph<T> = Csr<T, u64, Directed, NodeIndex>;

type IndexMap = HashMap<usize, NodeIndex>;


pub struct StorageCSRGraph<T>
    where
        T: Copy + Clone + Default
{
    root_index: Option<NodeIndex>,
    graph: CsrGraph<T>,
    node_map: HashMap<NodeIndex, T>,
    index_map: IndexMap,
}


impl<T> StorageCSRGraph<T>
    where
        T: Copy + Clone + Default
{
    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            root_index: None,
            graph: CsrGraph::with_nodes(capacity),
            node_map: HashMap::with_capacity(capacity),
            index_map: HashMap::with_capacity(capacity),
        }
    }
}

impl<T> Default for StorageCSRGraph<T>
    where
        T: Copy + Clone + Default
{
    fn default() -> Self {
        Self::new_with_capacity(100)
    }
}


impl<T> GraphStorage<T> for StorageCSRGraph<T>
    where
        T: Copy + Clone + Default
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
}


impl<T> GraphRoot<T> for StorageCSRGraph<T>
    where
        T: Copy + Clone + Default
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
            return None;
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


impl<T> GraphLike<T> for StorageCSRGraph<T>
    where
        T: Copy + Clone + Default
{
    fn clear(&mut self) {
        self.graph.clear_edges();
        self.node_map.clear();
        self.index_map.clear();
        self.root_index = None;
    }

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

        // CSR DOES NOT have a way to remove a node...
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
        self.graph.contains_edge(*k, *l)
    }

    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError> {
        if !self.contains_node(a) {
            return Err(UltraGraphError("index a not found".into()));
        };

        if !self.contains_node(b) {
            return Err(UltraGraphError("index b not found".into()));
        };

        // CSR DOES NOT have a way to remove an edge...

        Ok(())
    }
}
