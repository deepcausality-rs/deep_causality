/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ConstTree;
use std::collections::VecDeque;
use std::sync::Arc;

impl<T> ConstTree<T> {
    /// Returns a reference to the value stored at the root of this tree.
    pub fn value(&self) -> &T {
        &self.node.value
    }

    /// Returns a slice containing the children of the root of this tree.
    pub fn children(&self) -> &[ConstTree<T>] {
        &self.node.children
    }

    /// Checks if two `ConstTree` handles point to the same underlying node allocation.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.node, &other.node)
    }

    /// Returns a specific child by its index, if it exists.
    pub fn get_child(&self, index: usize) -> Option<&ConstTree<T>> {
        self.node.children.get(index)
    }

    /// Checks if this tree node has any children.
    pub fn is_leaf(&self) -> bool {
        self.node.children.is_empty()
    }

    /// Returns the total number of nodes in the tree (including the root).
    /// This is an O(n) operation as it traverses the entire tree.
    pub fn size(&self) -> usize {
        self.iter_pre_order().count()
    }

    /// Returns the maximum depth of the tree. A leaf node has a depth of 1.
    /// This is an O(n) iterative operation that is robust against stack overflows.
    pub fn depth(&self) -> usize {
        let mut max_depth = 0;
        let mut queue = VecDeque::new();

        // Push the root node with depth 1 to start.
        queue.push_back((self, 1));

        while let Some((current_node, current_depth)) = queue.pop_front() {
            max_depth = max_depth.max(current_depth);
            for child in current_node.children() {
                queue.push_back((child, current_depth + 1));
            }
        }
        max_depth
    }

    /// Finds the first node that satisfies a predicate in pre-order traversal.
    ///
    /// # Arguments
    /// * `predicate`: A closure that returns `true` for the node being sought.
    ///
    /// # Returns
    /// An `Option` containing a reference to the found `ConstTree`, or `None`.
    pub fn find<P>(&self, predicate: P) -> Option<&ConstTree<T>>
    where
        P: Fn(&T) -> bool,
    {
        self.iter_nodes_pre_order()
            .find(|node| predicate(node.value()))
    }

    /// Returns an iterator over all nodes that satisfy a predicate in pre-order.
    pub fn find_all<P>(&self, predicate: P) -> impl Iterator<Item = &ConstTree<T>>
    where
        P: Fn(&T) -> bool,
    {
        self.iter_nodes_pre_order()
            .filter(move |node| predicate(node.value()))
    }
}
