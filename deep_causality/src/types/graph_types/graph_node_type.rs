/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
use std::fmt::{Display, Formatter};

use crate::prelude::*;
use crate::protocols::contextuable::{Datable, SpaceTemporal, Spatial, Temporal};

// Node type needs to be generic over S and T to allow
// for categories of spacial and temporal types.
// https://stackoverflow.com/questions/31123882/how-to-map-a-parametrized-enum-from-a-generic-type-to-another


// Add type constraints to the where clause so that S adhere to spatial trait requirements
// and to temporal trait requirements for T.

// Make sure that traits are re-implement with S and T as generic parameters,
// which then allows to implement those traits for existing node types.
// https://www.geeksforgeeks.org/rust-generic-traits/

// https://stackoverflow.com/questions/69173586/either-type-a-or-b-in-rust
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum NodeType<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,
{
    Datoid(D),
    Tempoid(T),
    Root(Root),
    Spaceiod(S),
    SpaceTempoid(ST),
}

impl<D, S, T, ST> NodeType<D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        T: Temporal,
        ST: SpaceTemporal,

{
    pub fn dataoid(&self) -> Option<&D> {
        if let NodeType::Datoid(b) = self {
            Some(b)
        } else {
            None
        }
    }
    pub fn tempoid(&self) -> Option<&T> {
        if let NodeType::Tempoid(b) = self {
            Some(b)
        } else {
            None
        }
    }
    pub fn root(&self) -> Option<&Root> {
        if let NodeType::Root(b) = self {
            Some(b)
        } else {
            None
        }
    }
    pub fn spaceiod(&self) -> Option<&S> {
        if let NodeType::Spaceiod(b) = self {
            Some(b)
        } else {
            None
        }
    }
    pub fn space_tempoid(&self) -> Option<&ST> {
        if let NodeType::SpaceTempoid(b) = self {
            Some(b)
        } else {
            None
        }
    }
}

impl<D, S, T, ST> Display for NodeType<D, S, T, ST>
    where
        D: Datable + Display,
        S: Spatial + Display,
        T: Temporal + Display,
        ST: SpaceTemporal + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Datoid(b) => write!(f, "Datoid: {}", b),
            NodeType::Tempoid(b) => write!(f, "Tempoid: {}", b),
            NodeType::Root(b) => write!(f, "Root: {}", b),
            NodeType::Spaceiod(b) => write!(f, "Spaceiod: {}", b),
            NodeType::SpaceTempoid(b) => write!(f, "SpaceTempoid: {}", b),
        }
    }
}
