/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::TopologyError;
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

mod base_topology;
mod clone;
mod display;
mod getters;
mod graph_topology;

/// Represents a simple undirected graph (nodes and edges).
/// Nodes are represented by `usize` indices.
/// The type parameter T represents metadata associated with each node.
#[derive(Debug, Clone, PartialEq)]
pub struct Graph<T> {
    /// Number of vertices in the graph.
    pub(crate) num_vertices: usize,
    /// Adjacency list: map from vertex index to a list of its neighbors.
    pub(crate) adjacencies: BTreeMap<usize, Vec<usize>>,
    /// Number of edges in the graph.
    pub(crate) num_edges: usize,
    /// Metadata associated with each node
    pub(crate) data: CausalTensor<T>,
    /// The Focus (Cursor) for Comonadic extraction
    pub(crate) cursor: usize,
}

impl<T> Graph<T>
where
    T: Default + Copy + Clone + PartialEq + Zero,
{
    /// Creates a new empty `Graph` with a specified number of vertices.
    pub fn new(num_vertices: usize, data: CausalTensor<T>, cursor: usize) -> Result<Self, TopologyError> {
        if num_vertices == 0 {
            return Err(TopologyError::InvalidInput(
                "Graph must have at least one vertex".to_string(),
            ));
        }

        if data.len() != num_vertices {
            return Err(TopologyError::InvalidInput(
                "Data size must match number of vertices".to_string(),
            ));
        }

        if cursor >= num_vertices {
            return Err(TopologyError::IndexOutOfBounds(
                "Initial cursor out of bounds for Graph".to_string(),
            ));
        }

        let mut adjacencies = BTreeMap::new();
        for i in 0..num_vertices {
            adjacencies.insert(i, Vec::new());
        }

        Ok(Self {
            num_vertices,
            adjacencies,
            num_edges: 0,
            data,
            cursor,
        })
    }

    /// Adds an edge between two vertices.
    /// Returns `Ok(true)` if the edge was added, `Ok(false)` if it already existed.
    /// Returns an error if vertices are out of bounds.
    pub fn add_edge(&mut self, u: usize, v: usize) -> Result<bool, TopologyError> {
        if u >= self.num_vertices || v >= self.num_vertices {
            return Err(TopologyError::GraphError(format!(
                "Vertex index out of bounds: num_vertices={}, u={}, v={}",
                self.num_vertices, u, v
            )));
        }

        // Ensure no self-loops in simple graphs unless explicitly allowed.
        if u == v {
            return Err(TopologyError::GraphError(
                "Self-loops are not allowed in this graph implementation".to_string(),
            ));
        }

        // Add edge (u,v)
        let u_neighbors = self.adjacencies.get_mut(&u).unwrap();
        if !u_neighbors.contains(&v) {
            u_neighbors.push(v);
            u_neighbors.sort_unstable(); // Keep neighbors sorted for consistent representation and faster lookup
            self.num_edges += 1;
            // For an undirected graph, also add (v,u)
            let v_neighbors = self.adjacencies.get_mut(&v).unwrap();
            v_neighbors.push(u);
            v_neighbors.sort_unstable();
            Ok(true)
        } else {
            Ok(false) // Edge already exists
        }
    }

    /// Checks if an edge exists between two vertices.
    pub fn has_edge(&self, u: usize, v: usize) -> Result<bool, TopologyError> {
        if u >= self.num_vertices || v >= self.num_vertices {
            return Err(TopologyError::GraphError(format!(
                "Vertex index out of bounds: num_vertices={}, u={}, v={}",
                self.num_vertices, u, v
            )));
        }
        Ok(self
            .adjacencies
            .get(&u)
            .map_or(false, |neighbors| neighbors.contains(&v)))
    }

    /// Returns a reference to the neighbors of a given vertex.
    pub fn neighbors(&self, u: usize) -> Result<&Vec<usize>, TopologyError> {
        if u >= self.num_vertices {
            return Err(TopologyError::GraphError(format!(
                "Vertex index out of bounds: num_vertices={}, u={}",
                self.num_vertices, u
            )));
        }
        Ok(self.adjacencies.get(&u).unwrap())
    }

    /// Returns the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}
