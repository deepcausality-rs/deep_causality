// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use petgraph::Directed;
use petgraph::matrix_graph::MatrixGraph;
use crate::prelude::{Contextuable, Contextoid, Datable, NodeIndex, SpaceTemporal, Spatial, Temporal, RelationKind, Identifiable};

// Edge weights need to be numerical (u64) to make shortest path algo work.
type CtxGraph<'l, D, S, T, ST> = MatrixGraph<Contextoid<D, S, T, ST>, u64, Directed, Option<u64>, u32>;
// Preferably, hashmap should hold only a reference to the contextoid in the graph,
// but this causes some problems with the borrow checker hence the value and clone requirement.
type CtxMap<'l, D, S, T, ST> = HashMap<NodeIndex, Contextoid<D, S, T, ST>>;
//
//
type IndexMap = HashMap<usize, NodeIndex>;

#[derive(Clone)]
pub struct Context<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
{
    id: u64,
    name: &'l str,
    graph: CtxGraph<'l, D, S, T, ST>,
    context_map: CtxMap<'l, D, S, T, ST>,
    index_map: IndexMap,
}


impl<'l, D, S, T, ST> Context<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
{
    pub fn new(
        id: u64,
        name: &'l str,
    )
        -> Self
    {
        Self {
            id,
            name,
            graph: MatrixGraph::default(),
            context_map: HashMap::new(),
            index_map: HashMap::new(),
        }
    }

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
            graph: MatrixGraph::default(),
            context_map: HashMap::with_capacity(capacity),
            index_map: HashMap::with_capacity(capacity),
        }
    }


    pub fn name(&self) -> &str {
        self.name
    }
}

impl<'l, D, S, T, ST> Identifiable for Context<'l, D, S, T, ST> where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone
{
    fn id(&self) -> u64 {
        self.id
    }
}

impl<'l, D, S, T, ST> Contextuable<'l, D, S, T, ST> for Context<'l, D, S, T, ST> where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone
{

    fn add_node(
        &mut self,
        value: Contextoid<D, S, T, ST>,
    )
        -> usize
    {
        let node_index = self.graph.add_node(value.clone());
        self.context_map.insert(node_index, value);
        self.index_map.insert(node_index.index(), node_index);

        node_index.index()
    }

    fn contains_node(
        &self,
        index: usize,
    )
        -> bool
    {
        let k = self.index_map.get(&index).expect("index not found");
        self.context_map.contains_key(k)
    }

    fn get_node(
        &self,
        index: usize,
    )
        -> Option<&Contextoid<D, S, T, ST>>
    {
        let k = self.index_map.get(&index).expect("index not found");
        self.context_map.get(&k)
    }

    fn remove_node(
        &mut self,
        index: usize,
    )
    {
        let k = self.index_map.get(&index).expect("index not found");
        self.graph.remove_node(*k);
        self.context_map.remove(&k);
    }

    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    )
    {
        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.add_edge(*k, *l, weight as u64);
    }

    fn contains_edge(
        &self,
        a: usize,
        b: usize,
    )
        -> bool
    {
        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");
        self.graph.has_edge(*k, *l)
    }

    fn remove_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> u64
    {
        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.remove_edge(*k, *l)
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
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
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
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl<'l, D, S, T, ST> Display for Context<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}