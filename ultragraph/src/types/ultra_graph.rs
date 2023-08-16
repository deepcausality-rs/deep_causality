// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

#![forbid(unsafe_code)]

use std::marker::PhantomData;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct UltraGraph<S, T>
    where
        T: Copy,
        S: GraphStorage<T>,
{
    storage: S,
    ty: PhantomData<T>,
}

impl<S, T> UltraGraph<S, T>
    where
        T: Copy,
        S: GraphStorage<T>,
{
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            ty: PhantomData,
        }
    }
}

impl<S, T> GraphStorage<T> for UltraGraph<S, T>
    where
        T: Copy,
        S: GraphStorage<T>,
{
    fn size(&self) -> usize {
        self.storage.size()
    }

    fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    fn number_nodes(&self) -> usize {
        self.storage.number_nodes()
    }

    fn number_edges(&self) -> usize {
        self.storage.number_edges()
    }

    fn get_all_nodes(&self) -> Vec<T> {
        self.storage.get_all_nodes()
    }

    fn get_all_edges(&self) -> Vec<(usize, usize)> {
        self.storage.get_all_edges()
    }

    fn clear(&mut self) {
        self.storage.clear()
    }
}

impl<S, T> GraphRoot<T> for UltraGraph<S, T>
    where
        T: Copy,
        S: GraphStorage<T>,
{
    fn add_root_node(&mut self, value: T) -> usize {
        self.storage.add_root_node(value)
    }

    fn contains_root_node(&self) -> bool {
        self.storage.contains_root_node()
    }

    fn get_root_node(&self) -> Option<&T> {
        self.storage.get_root_node()
    }

    fn get_root_index(&self) -> Option<usize> {
        self.storage.get_root_index()
    }

    fn get_last_index(&self) -> Result<usize, UltraGraphError> {
        self.storage.get_last_index()
    }
}

impl<S, T> GraphLike<T> for UltraGraph<S, T>
    where
        T: Copy,
        S: GraphStorage<T>,
{
    fn add_node(&mut self, value: T) -> usize {
        self.storage.add_node(value)
    }

    fn contains_node(&self, index: usize) -> bool {
        self.storage.contains_node(index)
    }

    fn get_node(&self, index: usize) -> Option<&T> {
        self.storage.get_node(index)
    }

    fn remove_node(&mut self, index: usize) -> Result<(), UltraGraphError> {
        self.storage.remove_node(index)
    }

    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError> {
        self.storage.add_edge(a, b)
    }

    fn add_edge_with_weight(&mut self, a: usize, b: usize, weight: u64) -> Result<(), UltraGraphError> {
        self.storage.add_edge_with_weight(a, b, weight)
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.storage.contains_edge(a, b)
    }

    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError> {
        self.storage.remove_edge(a, b)
    }

    fn shortest_path(&self, start_index: usize, stop_index: usize) -> Result<Vec<usize>, UltraGraphError> {
        self.storage.shortest_path(start_index, stop_index)
    }

    fn outgoing_edges(&self, a: usize) -> Result<Vec<usize>, UltraGraphError> {
        self.storage.outgoing_edges(a)
    }
}
