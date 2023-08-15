// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Debug, Display, Formatter};

use ultragraph::prelude::*;

use crate::errors::ContextIndexError;
use crate::prelude::{Contextoid, ContextuableGraph, Datable, Identifiable, RelationKind, SpaceTemporal, Spatial, Temporal};

type CtxGraph<'l, D, S, T, ST> = UltraGraph<StorageMatrixGraph<Contextoid<D, S, T, ST>>, Contextoid<D, S, T, ST>>;

pub struct Context<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    id: u64,
    name: &'l str,
    graph: CtxGraph<'l, D, S, T, ST>,
}


impl<'l, D, S, T, ST> Context<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    /// Creates a new context with the given node capacity.
    pub fn with_capacity(
        id: u64,
        name: &'l str,
        capacity: usize,
    )
        -> Self
    {
        Self {
            id,
            name,
            graph: ultragraph::new_with_matrix_storage(capacity),
        }
    }

    /// Returns the name of the context.
    pub fn name(&self) -> &str {
        self.name
    }
}

impl<'l, D, S, T, ST> Identifiable for Context<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    /// Returns the id of the context.
    fn id(&self) -> u64 {
        self.id
    }
}

impl<'l, D, S, T, ST> ContextuableGraph<'l, D, S, T, ST> for Context<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    /// Ads a new Contextoid to the context.
    /// You can add the same contextoid multiple times,
    /// but each one will return a new and unique node index.
    fn add_node(
        &mut self,
        value: Contextoid<D, S, T, ST>,
    )
        -> usize
    {
        self.graph.add_node(value)
    }

    /// Returns only true if the context contains the contextoid with the given index.
    fn contains_node(
        &self,
        index: usize,
    )
        -> bool
    {
        self.graph.contains_node(index)
    }

    /// Returns a reference to the contextoid with the given index.
    /// If the context does not contain the contextoid, it will return None.
    fn get_node(
        &self,
        index: usize,
    )
        -> Option<&Contextoid<D, S, T, ST>>
    {
        self.graph.get_node(index)
    }

    /// Removes a contextoid from the context.
    /// Returns ContextIndexError if the index is not found
    fn remove_node(
        &mut self,
        index: usize,
    )
        -> Result<(), ContextIndexError>
    {
        if !self.contains_node(index) {
            return Err(ContextIndexError(format!("index {} not found", index)));
        };

        if self.graph.remove_node(index).is_err() {
            return Err(ContextIndexError(format!("index {} not found", index)));
        };

        Ok(())
    }

    /// Adds a new weighted edge between two nodes.
    /// Returns either Ok after success, or ContextIndexError if
    /// any of the nodes are not in the context.
    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    )
        -> Result<(), ContextIndexError>
    {
        if !self.contains_node(a) {
            return Err(ContextIndexError(format!("index a {} not found", a)));
        };

        if !self.contains_node(b) {
            return Err(ContextIndexError(format!("index b {} not found", b)));
        };

        if self.graph.add_edge_with_weight(a, b, weight as u64).is_err() {
            return Err(ContextIndexError(format!("Failed to add edge for index a {} and b {}", a, b)));
        }

        Ok(())
    }

    /// Returns only true if the context contains the edge between the two nodes.
    /// If the context does not contain the edge or any of the nodes it will return false.
    /// You may want to call contains_node first to ascertain that the nodes are in the context.
    fn contains_edge(
        &self,
        a: usize,
        b: usize,
    )
        -> bool
    {
        self.graph.contains_edge(a, b)
    }

    /// Removes an edge between two nodes.
    /// Returns either Ok after success, or ContextIndexError if
    /// any of the nodes are not in the context.
    fn remove_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> Result<(), ContextIndexError>
    {
        if !self.contains_node(a) {
            return Err(ContextIndexError("index a not found".into()));
        };

        if !self.contains_node(b) {
            return Err(ContextIndexError("index b not found".into()));
        };

        if self.graph.remove_edge(a, b).is_err() {
            return Err(ContextIndexError(format!("Failed to remove edge for index a {} and b {}", a, b)));
        }

        Ok(())
    }

    /// Returns the number of nodes in the context. Alias for node_count().
    fn size(
        &self
    )
        -> usize
    {
        self.graph.size()
    }

    /// Returns true if the context contains no nodes.
    fn is_empty(
        &self
    )
        -> bool
    {
        self.graph.is_empty()
    }

    /// Returns the number of nodes in the context.
    fn node_count(
        &self
    )
        -> usize
    {
        self.graph.number_nodes()
    }

    /// Returns the number of edges in the context.
    fn edge_count(
        &self
    )
        -> usize
    {
        self.graph.number_edges()
    }
}

impl<'l, D, S, T, ST> Context<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn format(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "Context: id: {}, name: {}, node_count: {}, edge_count: {}",
               self.id,
               self.name,
               self.node_count(),
               self.edge_count(),
        )
    }
}

impl<'l, D, S, T, ST> Debug for Context<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl<'l, D, S, T, ST> Display for Context<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}