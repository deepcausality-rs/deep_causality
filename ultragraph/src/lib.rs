// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

#![forbid(unsafe_code)]

use crate::prelude::{StorageCSRGraph, StorageMatrixGraph, UltraGraph};

pub mod prelude;
pub mod protocols;
pub mod errors;
pub mod storage;
pub mod types;

pub fn new_matrix_storage_with_capacity<T>(
    capacity: usize
)
    -> UltraGraph<StorageMatrixGraph<T>, T>
    where
        T: Copy + Clone + Default
{
    UltraGraph::new(StorageMatrixGraph::<T>::new_with_capacity(capacity))
}

pub fn new_csr_storage_with_capacity<T>(
    capacity: usize
)
    -> UltraGraph<StorageCSRGraph<T>, T>
    where
        T: Copy + Clone + Default
{
    UltraGraph::new(StorageCSRGraph::<T>::new_with_capacity(capacity))
}