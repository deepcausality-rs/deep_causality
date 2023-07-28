// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Display, Formatter};
use crate::prelude::NodeType;
use crate::protocols::contextuable::{Datable, SpaceTemporal, Spatial, Temporal};
use crate::protocols::identifiable::Identifiable;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Contextoid<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal, {
    id: u64,
    vertex_type: NodeType<D, S, T, ST>,
}

impl<D, S, T, ST> Contextoid<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    pub fn new(id: u64, vertex_type: NodeType<D, S, T, ST>) -> Self {
        Self { id, vertex_type }
    }
    pub fn vertex_type(&self) -> &NodeType<D, S, T, ST> {
        &self.vertex_type
    }
}

impl<D, S, T, ST> Identifiable for Contextoid<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    fn id(&self) -> u64 {
        self.id
    }
}

impl<D, S, T, ST> Display for Contextoid<D, S, T, ST>
    where
        D: Datable + Display,
        S: Spatial + Display,
        T: Temporal + Display,
        ST: SpaceTemporal + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vertex ID: {} Type: {}",
               self.id,
               self.vertex_type
        )
    }
}
