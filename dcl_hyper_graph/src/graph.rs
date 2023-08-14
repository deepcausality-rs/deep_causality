// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::cell::RefCell;
use std::fmt::Error;
use std::marker::PhantomData;

use crate::graph_like::GraphLike;
use crate::storage::Storage;

#[derive(Debug, Clone)]
pub struct HyperGraph<S, T>
    where
        T: Copy,
        S: Storage<T>,
{
    // interior mutability https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
    storage: RefCell<S>,
    ty: PhantomData<T>,
}

impl<S, T> HyperGraph<S, T>
    where
        T: Copy + Default,
        S: Storage<T>,
{
    pub fn new(storage: S) -> Self {
        Self {
            storage: RefCell::new(storage),
            ty: Default::default(),
        }
    }
}

impl<S, T> GraphLike<T> for HyperGraph<S, T>
    where
        T: Copy + Default,
        S: Storage<T>,
{
    fn clear_graph(&mut self) {
        self.storage.borrow_mut().clear_graph()
    }

    fn add_node(&mut self, value: T) -> usize {
        self.storage.borrow_mut().add_node(value)
    }

    fn contains_node(&self, index: usize) -> bool {
        self.storage.borrow().contains_node(index)
    }

    fn get_node(&self, index: usize) -> Option<T> {
        self.storage.borrow().get_node(index)
    }

    fn remove_node(&mut self, a: usize) -> T {
        self.storage.borrow_mut().remove_node(a)
    }

    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), Error> {
        self.storage.borrow_mut().add_edge(a, b)
    }

    fn add_edge_with_weight(&mut self, a: usize, b: usize, weight: u64) -> Result<(), Error> {
        self.storage.borrow_mut().add_edge_with_weight(a, b, weight)
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.storage.borrow().contains_edge(a, b)
    }

    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), Error> {
        self.storage.borrow_mut().remove_edge(a, b)
    }
}

impl<S, T> Storage<T> for HyperGraph<S, T>
    where
        T: Copy + Default,
        S: Storage<T>,
{
    fn size(&self) -> usize {
        self.storage.borrow().size()
    }

    fn is_empty(&self) -> bool {
        self.storage.borrow().is_empty()
    }

    fn number_nodes(&self) -> usize {
        self.storage.borrow().number_nodes()
    }

    fn number_edges(&self) -> usize {
        self.storage.borrow().number_edges()
    }
}