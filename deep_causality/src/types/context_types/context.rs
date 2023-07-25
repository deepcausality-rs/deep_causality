// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use std::fmt::{Debug, Display, Formatter};
use crate::prelude::{ContextMatrixGraph};
use crate::protocols::contextuable::{Datable, SpaceTemporal, Spatial, Temporal};

pub struct Context<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal
{
    id: u64,
    name: String,
    graph: ContextMatrixGraph<D, S, T, ST>,
}


impl<D, S, T, ST> Context<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal
{
    pub fn new(
        id: u64,
        name: String,
        graph: ContextMatrixGraph<D, S, T, ST>,
    )
        -> Self
    {
        Self { id, name, graph }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

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

impl<D, S, T, ST> Debug for Context<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl<D, S, T, ST> Display for Context<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}