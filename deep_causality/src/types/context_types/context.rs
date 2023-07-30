// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use petgraph::Directed;
use petgraph::matrix_graph::MatrixGraph;
use crate::prelude::{Contextuable, Contextoid, Datable, NodeIndex, SpaceTemporal, Spatial, Temporal, RelationKind};

// Edge weights need to be numerical (u64) to make shortest path algo work.
type CtxGraph<'l, D, S, T, ST> = MatrixGraph<Contextoid<D, S, T, ST>, u64, Directed, Option<u64>, u32>;
type CtxMap<'l, D, S, T, ST> = HashMap<NodeIndex,Contextoid<D, S, T, ST>>;

#[derive(Clone)]
pub struct Context<'l, D, S, T, ST>
    where
        D: Datable+Clone,
        S: Spatial+Clone,
        T: Temporal+Clone,
        ST: SpaceTemporal+Clone
{
    id: u64,
    name: &'l str,
    graph: CtxGraph<'l, D, S, T, ST>,
    context_map: CtxMap<'l, D, S, T, ST>,
}


impl<'l, D, S, T, ST> Context<'l, D, S, T, ST>
    where
        D: Datable+Clone,
        S: Spatial+Clone,
        T: Temporal+Clone,
        ST: SpaceTemporal+Clone
{
    pub fn new(
        id: u64,
        name:  &'l str,
    )
        -> Self
    {
        Self { id, name, graph: MatrixGraph::default(), context_map: HashMap::new() }
    }

    pub fn with_capacity(
        id: u64,
        name:  &'l str,
        capacity: usize,
    )
        -> Self
    {
        Self { id, name, graph: MatrixGraph::default(), context_map: HashMap::with_capacity(capacity) }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name
    }
}


impl<'l, D, S, T, ST> Contextuable<'l, D, S, T, ST> for Context<'l, D, S, T, ST> where
    D: Datable+Clone,
    S: Spatial+Clone,
    T: Temporal+Clone,
    ST: SpaceTemporal+Clone
{
    fn add_node(
        &mut self,
        value: Contextoid<D, S, T, ST>,
    )
        -> NodeIndex
    {

        let node_index = self.graph.add_node(value.clone());
        self.context_map.insert(node_index, value);

        node_index
    }

    fn contains_node(
        &self,
        index: NodeIndex,
    )
        -> bool
    {
        self.context_map.contains_key(&index)
    }

    fn get_node(
        &self,
        index: NodeIndex,
    )
        -> Option<&Contextoid<D, S, T, ST>>
    {
        self.context_map.get(&index)
    }
    fn remove_node(
        &mut self,
        index: NodeIndex,
    )
    {
        self.graph.remove_node(index);
        self.context_map.remove(&index);
    }

    fn add_edge(
        &mut self,
        a: NodeIndex,
        b: NodeIndex,
        weight: RelationKind,
    )
    {
        self.graph.add_edge(a, b, weight as u64);
    }

    fn contains_edge(
        &self,
        a: NodeIndex,
        b: NodeIndex,
    )
        -> bool
    {
        self.graph.has_edge(a, b)
    }

    fn remove_edge(
        &mut self,
        a: NodeIndex,
        b: NodeIndex,
    )
        -> u64
    {
        self.graph.remove_edge(a, b)
    }

    fn size(
        &self
    )
        -> usize
    {
        self.context_map.len()
    }
    fn is_empty(
        &self
    )
        -> bool
    {
        self.context_map.is_empty()
    }

    fn node_count(
        &self
    )
        -> usize
    {
        self.graph.node_count()
    }

    fn edge_count(
        &self
    )
        -> usize
    {
        self.graph.edge_count()
    }
}

impl<'l, D, S, T, ST> Context<'l, D, S, T, ST>
    where
        D: Datable+Clone,
        S: Spatial+Clone,
        T: Temporal+Clone,
        ST: SpaceTemporal+Clone
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
        D: Datable+Clone,
        S: Spatial+Clone,
        T: Temporal+Clone,
        ST: SpaceTemporal+Clone
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl<'l, D, S, T, ST> Display for Context<'l, D, S, T, ST>
    where
        D: Datable+Clone,
        S: Spatial+Clone,
        T: Temporal+Clone,
        ST: SpaceTemporal+Clone
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}