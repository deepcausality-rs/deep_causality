// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::marker::PhantomData;

use crate::error::HyperGraphError;
use crate::graph_like::GraphLike;
use crate::storage::Storage;

#[derive(Debug, Clone)]
pub struct UltraGraph<S, T>
    where
        T: Copy,
        S: Storage<T>,
{
    storage: S,
    ty: PhantomData<T>,
}

impl<S, T> UltraGraph<S, T>
    where
        T: Copy + Default,
        S: Storage<T>,
{
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            ty: Default::default(),
        }
    }
}

impl<S, T> GraphLike<T> for UltraGraph<S, T>
    where
        T: Copy + Default,
        S: Storage<T>,
{
    fn clear_graph(&mut self) {
        self.storage.clear_graph()
    }

    fn add_node(&mut self, value: T) -> usize {
        self.storage.add_node(value)
    }

    fn contains_node(&self, index: usize) -> bool {
        self.storage.contains_node(index)
    }

    fn get_node(&self, index: usize) -> Option<&T> {
        self.storage.get_node(index)
    }

    fn remove_node(&mut self, index: usize) -> Result<(), HyperGraphError> {
        self.storage.remove_node(index)
    }

    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), HyperGraphError> {
        self.storage.add_edge(a, b)
    }

    fn add_edge_with_weight(&mut self, a: usize, b: usize, weight: u64) -> Result<(), HyperGraphError> {
        self.storage.add_edge_with_weight(a, b, weight)
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.storage.contains_edge(a, b)
    }

    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), HyperGraphError> {
        self.storage.remove_edge(a, b)
    }
}

impl<S, T> Storage<T> for UltraGraph<S, T>
    where
        T: Copy + Default,
        S: Storage<T>,
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
}