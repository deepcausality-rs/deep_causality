/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */


use crate::prelude::{ContextKind, ContextMatrixGraph, Datable, SpaceTemporal, Spatial, Temporal};

pub struct ContextGraph<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal
{
    id: u64,
    name: String,
    kind: ContextKind,
    graph: ContextMatrixGraph<D, S, T, ST>,
}


impl<D, S, T, ST> ContextGraph<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal
{
    pub fn new(
        id: u64,
        name: String,
        kind: ContextKind,
        graph: ContextMatrixGraph<D, S, T, ST>,
    )
        -> Self
    {
        Self { id, name, kind, graph }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &ContextKind {
        &self.kind
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}