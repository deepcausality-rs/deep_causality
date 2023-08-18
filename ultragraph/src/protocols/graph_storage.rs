// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::protocols::graph_like::GraphLike;
use crate::protocols::graph_root::GraphRoot;

pub trait GraphStorage<T>: GraphLike<T> + GraphRoot<T>
{
    fn size(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn number_nodes(&self) -> usize;

    fn number_edges(&self) -> usize;

    fn get_all_nodes(&self) -> Vec<&T>;

    fn get_all_edges(&self) -> Vec<(usize, usize)>;

    fn clear(&mut self);
}