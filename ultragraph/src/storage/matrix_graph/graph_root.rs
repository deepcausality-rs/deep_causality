// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::errors::UltraGraphError;
use crate::prelude::{GraphLike, GraphRoot, GraphStorage};

use super::{NodeIndex, UltraMatrixGraph};

impl<T> GraphRoot<T> for UltraMatrixGraph<T>
{
    fn add_root_node(
        &mut self,
        value: T,
    )
        -> usize
    {
        let idx = self.add_node(value);
        let root_index = NodeIndex::new(idx);
        self.root_index = Some(root_index);
        self.index_map.insert(root_index.index(), root_index);
        root_index.index()
    }

    fn contains_root_node(
        &self
    )
        -> bool
    {
        self.root_index.is_some()
    }

    fn get_root_node(
        &self
    )
        -> Option<&T>
    {
        if self.contains_root_node()
        {
            self.node_map.get(&self.root_index.unwrap())
        } else {
            None
        }
    }

    fn get_root_index(
        &self
    )
        -> Option<usize>
    {
        if self.contains_root_node() {
            Some(self.root_index.unwrap().index())
        } else {
            None
        }
    }

    fn get_last_index(
        &self
    )
        -> Result<usize, UltraGraphError>
    {
        if !self.is_empty() {
            Ok(self.node_map.len())
        } else {
            Err(UltraGraphError("Graph is empty".to_string()))
        }
    }
}
