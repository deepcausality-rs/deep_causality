/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::UltraGraphError;
use crate::prelude::GraphLike;

use super::UltraMatrixGraph;

impl<T> GraphLike<T> for UltraMatrixGraph<T> {
    fn add_node(&mut self, value: T) -> usize {
        let node_index = self.graph.add_node(true);
        self.node_map.insert(node_index, value);
        self.index_map.insert(node_index.index(), node_index);
        node_index.index()
    }

    fn contains_node(&self, index: usize) -> bool {
        self.index_map.get(&index).is_some()
    }

    fn get_node(&self, index: usize) -> Option<&T> {
        if !self.contains_node(index) {
            None
        } else {
            let k = self.index_map.get(&index).expect("index not found");
            self.node_map.get(k)
        }
    }

    fn remove_node(&mut self, index: usize) -> Result<(), UltraGraphError> {
        if !self.contains_node(index) {
            return Err(UltraGraphError(format!("index {index} not found")));
        };

        // Check if the node to be removed is the root node.
        if let Some(root_node_index) = self.root_index {
            if root_node_index.index() == index {
                // If so, clear the root index.
                self.root_index = None;
            }
        }

        let k = self.index_map.get(&index).unwrap();
        self.graph.remove_node(*k);
        self.node_map.remove(k);
        self.index_map.remove(&k.index());
        Ok(())
    }

    fn update_node(&mut self, index: usize, value: T) -> Result<(), UltraGraphError> {
        // 1. Find the internal, stable NodeIndex using the public-facing usize index.
        let node_index = *self.index_map.get(&index).ok_or_else(|| {
            UltraGraphError(format!("update_node failed: index {index} not found"))
        })?;

        // 2. Update the payload in the node_map.
        // The `insert` method on a HashMap updates the value if the key exists.
        // This operation does not touch the `self.graph` field, preserving all edges.
        if self.node_map.insert(node_index, value).is_some() {
            // The key existed and the value was updated.
            Ok(())
        } else {
            // This is a consistency error: the index was in index_map but not node_map.
            // This should be unreachable if the graph is consistent, but we handle it defensively.
            Err(UltraGraphError(format!(
                "update_node failed: inconsistent state for index {index}"
            )))
        }
    }

    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError> {
        if !self.contains_node(a) {
            return Err(UltraGraphError(format!("index a {a} not found")));
        };

        if !self.contains_node(b) {
            return Err(UltraGraphError(format!("index b {b} not found")));
        };

        if self.contains_edge(a, b) {
            return Err(UltraGraphError(format!(
                "Edge already exists between: {a} and {b}"
            )));
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
    ) -> Result<(), UltraGraphError> {
        if !self.contains_node(a) {
            return Err(UltraGraphError(format!("index a {a} not found")));
        };

        if !self.contains_node(b) {
            return Err(UltraGraphError(format!("index b {b} not found")));
        };

        if self.contains_edge(a, b) {
            return Err(UltraGraphError(format!(
                "Edge already exists between: {a} and {b}"
            )));
        }

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");
        self.graph.add_edge(*k, *l, weight);
        Ok(())
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool {
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

        if !self.contains_edge(a, b) {
            return Err(UltraGraphError(format!(
                "Edge does not exist between: {a} and {b}"
            )));
        }

        // Because of the `contains_node` checks above, these `expect` calls
        // are safe. They would only panic if there were an internal consistency
        // bug within UltraMatrixGraph itself, which is an appropriate use of expect.
        let k = self
            .index_map
            .get(&a)
            .expect("Inconsistent state: node a not in index_map");
        let l = self
            .index_map
            .get(&b)
            .expect("Inconsistent state: node b not in index_map");

        // This call to the panicky petgraph API is now SAFE because of our `contains_edge` check.
        // We can simply call the function and ignore its return value (the old edge weight),
        // as our function's contract is only to remove the edge and return Ok(()).
        self.graph.remove_edge(*k, *l);

        Ok(())
    }
}
