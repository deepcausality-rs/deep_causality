// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::*;

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
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ContextoidType<D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    Datoid(D),
    Tempoid(T),
    Root(Root),
    Spaceoid(S),
    SpaceTempoid(ST),
    _Unreachable(PhantomData<V>),
}

impl<D, S, T, ST, V> ContextoidType<D, S, T, ST, V>
where
    D: Datable,
    S: Spatial<V>,
    T: Temporable<V>,
    ST: SpaceTemporal<V>,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
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
    pub fn spaceiod(&self) -> Option<&S> {
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
}

impl<D, S, T, ST, V> Display for ContextoidType<D, S, T, ST, V>
where
    D: Display + Datable,
    S: Display + Spatial<V>,
    T: Display + Temporable<V>,
    ST: Display + SpaceTemporal<V>,
    V: Display
        + Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextoidType::Datoid(b) => write!(f, "Datoid: {}", b),
            ContextoidType::Tempoid(b) => write!(f, "Tempoid: {}", b),
            ContextoidType::Root(b) => write!(f, "Root: {}", b),
            ContextoidType::Spaceoid(b) => write!(f, "Spaceiod: {}", b),
            ContextoidType::SpaceTempoid(b) => write!(f, "SpaceTempoid: {}", b),
            _ => write!(f, ""),
        }
    }
}
