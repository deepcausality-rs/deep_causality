// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use petgraph::Directed;
use petgraph::graph::{NodeIndex as GraphNodeIndex};
use petgraph::matrix_graph::MatrixGraph;
use crate::errors::ContextIndexError;
use crate::prelude::{ContextuableGraph, Contextoid, Datable, SpaceTemporal, Spatial, Temporal, RelationKind, Identifiable};

type DefaultIx = u32;
type NodeIndex<Ix = DefaultIx> = GraphNodeIndex<Ix>;

//
// Edge weights need to be numerical (u64) to make shortest path algo work.
// Also, u32 is used as node node index type to bypass the fairly ancient 65k node limit
// coming from the u16 default node index default type in petgraph.
// u32 has a limit of 2^31 - 1 (4,294,967,295). NodeIndex can be at most u32 because petgraph has no implementation
// for u64 or u128. See: https://docs.rs/petgraph/latest/petgraph/graph/trait.IndexType.html
type CtxGraph<'l, D, S, T, ST> = MatrixGraph<Contextoid<D, S, T, ST>, u64, Directed, Option<u64>, u32>;
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
type CtxMap<'l, D, S, T, ST> = HashMap<NodeIndex, Contextoid<D, S, T, ST>>;
//
// There is a weirdo bug in the petgraph crate so that if you try to access a node of the graph
// with a newly constructed NodeIndex, it always returns None. However, if you use the NodeIndex
// returned from the add node method, it works.
//
// let idx NodeIndex::new(0)
// graph.get(idx) // returns None
//
// let idx = graph.add_node(Contextoid::<D, S, T, ST>::default()); // idx 0
// graph.get(idx) // returns Some
//
// NodeIndex is an alias:
// pub type NodeIndex<Ix = DefaultIx> = GraphNodeIndex<Ix>;
//
// Given that the CtxGraph overwrites the default (u16) NodeIndex type with u32 to bypass
// the 65k node limit (#547 https://github.com/petgraph/petgraph/pull/547),
// it might be possible that the NodeIndex may still leans on the internal default U16 index type
// and hence generates incompatible indices.
//
// Theoretically, you can override the NodeIndex with a custom type alias and re-export it,
// but in general, you don't really want to expose the internal graph index type through the API.
// Therefore I added an internal map that literally only maps usize to NodeIndex.
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
    /// Creates a new context with the given name and id.
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
            graph: MatrixGraph::default(),
            context_map: HashMap::with_capacity(capacity),
            index_map: HashMap::with_capacity(capacity),
        }
    }

    /// Returns the name of the context.
    pub fn name(&self) -> &str {
        self.name
    }
}

impl<'l, D, S, T, ST> Identifiable for Context<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
{
    /// Returns the id of the context.
    fn id(&self) -> u64 {
        self.id
    }
}

impl<'l, D, S, T, ST> ContextuableGraph<'l, D, S, T, ST> for Context<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone
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
        let node_index = self.graph.add_node(value.clone());
        self.context_map.insert(node_index, value);
        self.index_map.insert(node_index.index(), node_index);

        node_index.index()
    }

    /// Returns only true if the context contains the contextoid with the given index.
    fn contains_node(
        &self,
        index: usize,
    )
        -> bool
    {
        self.index_map.get(&index).is_some()
    }

    /// Returns a reference to the contextoid with the given index.
    /// If the context does not contain the contextoid, it will return None.
    fn get_node(
        &self,
        index: usize,
    )
        -> Option<&Contextoid<D, S, T, ST>>
    {
        return if !self.contains_node(index) {
            None
        } else {
            let k = self.index_map.get(&index).expect("index not found");
            self.context_map.get(k)
        };
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

        let k = self.index_map.get(&index).unwrap();
        self.graph.remove_node(*k);
        self.context_map.remove(k);

        self.index_map.remove(&index);

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

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.add_edge(*k, *l, weight as u64);

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
        if !self.contains_node(a) {
            return false;
        };

        if !self.contains_node(b) {
            return false;
        };

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.has_edge(*k, *l)
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

        let k = self.index_map.get(&a).expect("index not found");
        let l = self.index_map.get(&b).expect("index not found");

        self.graph.remove_edge(*k, *l);

        Ok(())
    }

    /// Returns the number of nodes in the context. Alias for node_count().
    fn size(
        &self
    )
        -> usize
    {
        self.context_map.len()
    }

    /// Returns true if the context contains no nodes.
    fn is_empty(
        &self
    )
        -> bool
    {
        self.context_map.is_empty()
    }

    /// Returns the number of nodes in the context.
    fn node_count(
        &self
    )
        -> usize
    {
        self.graph.node_count()
    }

    /// Returns the number of edges in the context.
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