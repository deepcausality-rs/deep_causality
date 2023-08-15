// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::errors::UltraGraphError;
use crate::protocols::graph_like::GraphLike;

pub trait GraphRoot<T>: GraphLike<T>
    where
        T: Copy,
{
    fn add_root_node(&mut self, value: T) -> usize;
    fn contains_root_node(&self) -> bool;
    fn get_root_node(&self) -> Option<&T>;
    fn get_root_index(&self) -> Option<usize>;
    fn get_last_index(&self) -> Result<usize, UltraGraphError>;
}
