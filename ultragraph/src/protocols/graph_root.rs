// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::errors::UltraGraphError;
use crate::protocols::graph_like::GraphLike;

pub trait GraphRoot<T>: GraphLike<T> {
    fn add_root_node(&mut self, value: T) -> usize;
    fn contains_root_node(&self) -> bool;
    fn get_root_node(&self) -> Option<&T>;
    fn get_root_index(&self) -> Option<usize>;
    fn get_last_index(&self) -> Result<usize, UltraGraphError>;
}
