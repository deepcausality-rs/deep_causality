// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

#![forbid(unsafe_code)]

use crate::prelude::{StorageMatrixGraph, UltraGraph};

pub mod prelude;
pub mod ultra_graph;
pub mod protocols;
pub mod errors;
pub mod storage;

pub fn new<T>()
    -> UltraGraph<StorageMatrixGraph<T>, T>
    where
        T: Copy + Clone + Default
{
    UltraGraph::new(StorageMatrixGraph::<T>::new())
}

pub fn new_with_capacity<T>(
    capacity: usize
)
    -> UltraGraph<StorageMatrixGraph<T>, T>
    where
        T: Copy + Clone + Default
{
    UltraGraph::new(StorageMatrixGraph::<T>::new_with_capacity(capacity))
}