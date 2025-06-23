// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;

use crate::prelude::*;
// Node type needs to be generic over S and T to allow
// for categories of spacial and temporal types.
// https://stackoverflow.com/questions/31123882/how-to-map-a-parametrized-enum-from-a-generic-type-to-another

// Add type constraints to the where clause so that S adhere to spatial trait requirements
// and to temporal trait requirements for T.

// Make sure that traits are re-implement with S and T as generic parameters,
// which then allows to implement those traits for existing node types.
// https://www.geeksforgeeks.org/rust-generic-traits/

// https://stackoverflow.com/questions/69173586/either-type-a-or-b-in-rust

/// Enum of monoidal context node types (each a composable unit of structure).
/// Each variant name ends in `-oid` to emphasize its monoid role as a single identity-bearing unit.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ContextKind {
    Datoid,
    Tempoid,
    Root,
    Spaceoid,
    SpaceTempoid,
    Symboid,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ContextoidType<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    Datoid(D),
    Tempoid(T),
    Root(Root),
    Spaceoid(S),
    SpaceTempoid(ST),
    Symboid(SYM),
    #[doc(hidden)]
    _Marker(PhantomData<V>),
}

impl<D, S, T, ST, SYM, V> ContextoidType<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    pub fn kind(&self) -> ContextKind {
        match self {
            ContextoidType::Datoid(_) => ContextKind::Datoid,
            ContextoidType::Tempoid(_) => ContextKind::Tempoid,
            ContextoidType::Root(_) => ContextKind::Root,
            ContextoidType::Spaceoid(_) => ContextKind::Spaceoid,
            ContextoidType::SpaceTempoid(_) => ContextKind::SpaceTempoid,
            ContextoidType::Symboid(_) => ContextKind::Symboid,
            _ => unreachable!(), // phantom variant
        }
    }
}

impl<D, S, T, ST, SYM, V> ContextoidType<D, S, T, ST, SYM, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporal<V>,
    ST: SpaceTemporal<V>,
    SYM: Symbolic,
{
    pub fn root(&self) -> Option<&Root> {
        if let ContextoidType::Root(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn dataoid(&self) -> Option<&D> {
        if let ContextoidType::Datoid(b) = self {
            Some(b)
        } else {
            None
        }
    }
    pub fn tempoid(&self) -> Option<&T> {
        if let ContextoidType::Tempoid(b) = self {
            Some(b)
        } else {
            None
        }
    }
    pub fn spaceoid(&self) -> Option<&S> {
        if let ContextoidType::Spaceoid(b) = self {
            Some(b)
        } else {
            None
        }
    }
    pub fn space_tempoid(&self) -> Option<&ST> {
        if let ContextoidType::SpaceTempoid(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn symboid(&self) -> Option<&SYM> {
        if let ContextoidType::Symboid(b) = self {
            Some(b)
        } else {
            None
        }
    }
}

impl<D, S, T, ST, SYM, V> Display for ContextoidType<D, S, T, ST, SYM, V>
where
    D: Display + Datable,
    S: Display + Spatial<V>,
    T: Display + Temporal<V>,
    ST: Display + SpaceTemporal<V>,
    SYM: Display + Symbolic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextoidType::Datoid(b) => write!(f, "Datoid: {}", b),
            ContextoidType::Tempoid(b) => write!(f, "Tempoid: {}", b),
            ContextoidType::Root(b) => write!(f, "Root: {}", b),
            ContextoidType::Spaceoid(b) => write!(f, "Spaceoid: {}", b),
            ContextoidType::SpaceTempoid(b) => write!(f, "SpaceTempoid: {}", b),
            ContextoidType::Symboid(b) => write!(f, "Symboid: {}", b),
            ContextoidType::_Marker(_) => unreachable!("_Marker variant should never be accessed directly"),
        }
    }
}
