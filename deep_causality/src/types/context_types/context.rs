// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


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
}