/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ConstTree;
use crate::const_tree::Node;
use std::sync::Arc;

impl<T: Clone> ConstTree<T> {
    /// Returns a new `ConstTree` with a new child appended.
    pub fn add_child(&self, child: ConstTree<T>) -> Self {
        let mut new_children = self.node.children.clone();
        new_children.push(child);

        Self {
            node: Arc::new(Node {
                value: self.node.value.clone(),
                children: new_children,
            }),
        }
    }

    /// Returns a new `ConstTree` with the children replaced by a new set.
    pub fn replace_children(&self, new_children: impl IntoIterator<Item = Self>) -> Self {
        Self {
            node: Arc::new(Node {
                value: self.node.value.clone(),
                children: new_children.into_iter().collect(),
            }),
        }
    }

    /// Returns a new `ConstTree` with the child at `index` updated.
    /// Returns `None` if the index is out of bounds.
    pub fn update_child(&self, index: usize, new_child: ConstTree<T>) -> Option<Self> {
        if index >= self.node.children.len() {
            return None;
        }
        let mut new_children = self.node.children.clone();
        new_children[index] = new_child;
        Some(Self {
            node: Arc::new(Node {
                value: self.node.value.clone(),
                children: new_children,
            }),
        })
    }

    /// Returns a new `ConstTree` with the child at `index` removed.
    /// Returns `None` if the index is out of bounds.
    pub fn remove_child(&self, index: usize) -> Option<Self> {
        if index >= self.node.children.len() {
            return None;
        }
        let mut new_children = self.node.children.clone();
        new_children.remove(index);
        Some(Self {
            node: Arc::new(Node {
                value: self.node.value.clone(),
                children: new_children,
            }),
        })
    }
}

impl<T: PartialEq> ConstTree<T> {
    /// Checks if the tree contains a given value.
    ///
    /// The search is performed in pre-order.
    pub fn contains(&self, value: &T) -> bool {
        self.iter_pre_order().any(|v| v == value)
    }
}
