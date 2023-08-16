// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use crate::errors::UltraGraphError;

pub trait GraphLike<T>
    where
        T: Copy,
{
    fn add_node(&mut self, value: T) -> usize;

    fn contains_node(&self, index: usize) -> bool;

    fn get_node(&self, index: usize) -> Option<&T>;

    fn remove_node(&mut self, index: usize) -> Result<(), UltraGraphError>;

    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> Result<(), UltraGraphError>;

    fn add_edge_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    )
        -> Result<(), UltraGraphError>;

    fn contains_edge(
        &self,
        a: usize,
        b: usize,
    )
        -> bool;

    fn remove_edge(
        &mut self,
        a: usize,
        b: usize,
    )
        -> Result<(), UltraGraphError>;

    /// Returns the path of subsequent NodeId from start to finish, if one was found.
    fn shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    )
        -> Result<Vec<usize>, UltraGraphError>;

    /// Returns all nodes with an outgoing edge starting from a.
    fn outgoing_edges(
        &self,
        a: usize,
    )
        -> Result<Vec<usize>, UltraGraphError>;
}