// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::{Adjustable, Contextoid, Identifiable, NodeIndex, RelationKind, TimeScale};

pub trait Datable: Adjustable + Identifiable {}


pub trait Temporal: Identifiable + Adjustable {}


// Specializes the `Temporal` trait.
pub trait Temporable: Temporal
{
    fn time_scale(&self) -> TimeScale;
    fn time_unit(&self) -> u32;
}


pub trait Spatial: Identifiable + Adjustable
{
    fn x(&self) -> i64;
    fn y(&self) -> i64;
    fn z(&self) -> i64;
}


pub trait SpaceTemporal: Identifiable + Spatial + Temporal + Adjustable
{
    fn t(&self) -> u64; // returns 4th dimension, t
}


pub trait Contextuable<'l, D, S, T, ST>
    where
        D: Datable,
        S: Spatial,
        ST: SpaceTemporal,
        T: Temporal
{
    fn add_node(&mut self, value: &'l Contextoid<D, S, T, ST>) -> NodeIndex;
    fn contains_node(&self, index: NodeIndex) -> bool;
    fn get_node(&self, index: NodeIndex) -> Option<&&Contextoid<D, S, T, ST>>;
    fn remove_node(&mut self, index: NodeIndex);
    fn add_edge(&mut self, a: NodeIndex, b: NodeIndex, weight: RelationKind);
    fn contains_edge(&self, a: NodeIndex, b: NodeIndex) -> bool;
    fn remove_edge(&mut self, a: NodeIndex, b: NodeIndex) -> u64;
    fn size(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
}
