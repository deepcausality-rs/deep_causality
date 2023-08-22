// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::vec::IntoIter;

use petgraph::algo::astar;
use petgraph::prelude::EdgeRef;

use crate::errors::UltraGraphError;
use crate::prelude::GraphLike;

use super::{NodeIndex, UltraMatrixGraph};

impl<T> GraphLike<T> for UltraMatrixGraph<T>
{
    fn add_node(
        &mut self,
        value: T,
    )
        -> usize
    {
        let node_index = self.graph.add_node(true);
        self.node_map.insert(node_index, value);
        self.index_map.insert(node_index.index(), node_index);
        node_index.index()
    }

    fn contains_node(
        &self,
        index: usize,
    )
        -> bool
    {
        self.index_map.get(&index).is_some()
    }

    fn get_node(
        &self,
        index: usize,
    )
        -> Option<&T>
    {
        if !self.contains_node(index) {
            None
        } else {
            let k = self.index_map.get(&index).expect("index not found");
            self.node_map.get(k)
        }
    }

    fn remove_node(
        &mut self,
        index: usize,
    )
        -> Result<(), UltraGraphError>
    {
        if !self.contains_node(index) {
            return Err(UltraGraphError(format!("index {} not found", index)));
        };

        let k = self.index_map.get(&index).unwrap();
        self.graph.remove_node(*k);
        self.node_map.remove(k);
        self.index_map.remove(&k.index());
        Ok(())
    }

    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> Result<(), UltraGraphError>
    {
        if !self.contains_node(a) {
            return Err(UltraGraphError(format!("index a {} not found", a)));
        };

        if !self.contains_node(b) {
            return Err(UltraGraphError(format!("index b {} not found", b)));
        };

        if self.contains_edge(a, b) {
            return Err(UltraGraphError(format!("Edge already exists between: {} and {}", a, b)));
        }

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");
        self.graph.add_edge(*k, *l, 0);
        Ok(())
    }

    fn add_edge_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    )
        -> Result<(), UltraGraphError>
    {
        if !self.contains_node(a) {
            return Err(UltraGraphError(format!("index a {} not found", a)));
        };

        if !self.contains_node(b) {
            return Err(UltraGraphError(format!("index b {} not found", b)));
        };

        if self.contains_edge(a, b) {
            return Err(UltraGraphError(format!("Edge already exists between: {} and {}", a, b)));
        }

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");
        self.graph.add_edge(*k, *l, weight);
        Ok(())
    }

    fn contains_edge(
        &self,
        a: usize,
        b: usize,
    )
        -> bool
    {
        if !self.contains_node(a) || !self.contains_node(b) {
            return false;
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");
        self.graph.has_edge(*k, *l)
    }

    fn remove_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> Result<(), UltraGraphError>
    {
        if !self.contains_node(a) {
            return Err(UltraGraphError("index a not found".into()));
        };

        if !self.contains_node(b) {
            return Err(UltraGraphError("index b not found".into()));
        };

        if !self.contains_edge(a, b) {
            return Err(UltraGraphError(format!("Edge does not exists between: {} and {}", a, b)));
        }

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.remove_edge(*k, *l);
        self.index_map.remove(&a);
        self.index_map.remove(&b);

        Ok(())
    }

    fn shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    )
        -> Option<Vec<usize>>
    {
        if !self.contains_node(start_index) {
            return None;
        };

        if !self.contains_node(stop_index) {
            return None;
        };

        let mut result: Vec<usize> = Vec::new();

        // A* algorithm https://docs.rs/petgraph/latest/petgraph/algo/astar/fn.astar.html
        if let Some((_, path)) = astar(
            &self.graph,
            NodeIndex::new(start_index),
            |finish| finish == NodeIndex::new(stop_index),
            |e| *e.weight(),
            |_| 0)
        {
            for node in path {
                result.push(node.index());
            }
            Some(result)
        } else {
            None
        }
    }

    fn outgoing_edges(
        &self,
        a: usize,
    )
        -> Result<IntoIter<usize>, UltraGraphError>
    {
        if !self.contains_node(a) {
            return Err(UltraGraphError("index a not found".into()));
        };

        let mut result: Vec<usize> = Vec::new();

        let neighbors = self.graph.neighbors(NodeIndex::new(a));

        for node in neighbors {
            result.push(node.index());
        }

        Ok(result.into_iter())
    }
}
