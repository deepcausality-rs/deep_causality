// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

#![forbid(unsafe_code)]

use crate::prelude::{StorageCSRGraph, StorageMatrixGraph, UltraGraph};

pub mod prelude;
pub mod protocols;
pub mod errors;
pub mod storage;
pub mod types;

/// Returns a new UltraGraph with matrix storage backend.
///
/// # Arguments
/// * Capacity refers to the maximum number of nodes that fit into the graph before a resize occurs.
///
pub fn new_with_matrix_storage<T>(
    capacity: usize
)
    -> UltraGraph<StorageMatrixGraph<T>, T>
    where
        T: Copy + Clone + Default
{
    UltraGraph::new(StorageMatrixGraph::<T>::new_with_capacity(capacity))
}

/// Returns a new UltraGraph with a Compressed Sparse Representation (CSR) as storage backend.
/// APPEND ONLY UltraGraph!
/// CSR storage does not support the removal of nodes or edges.
///
/// # Arguments
/// * Capacity refers to the maximum number of nodes that fit into the graph before a resize occurs.
///
pub fn new_with_csr_storage<T>(
    capacity: usize
)
    -> UltraGraph<StorageCSRGraph<T>, T>
    where
        T: Copy + Clone + Default
{
    UltraGraph::new(StorageCSRGraph::<T>::new_with_capacity(capacity))
}