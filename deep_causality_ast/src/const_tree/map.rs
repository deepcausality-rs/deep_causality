/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ConstTree;
use std::sync::Arc;

impl<T: Clone> ConstTree<ConstTree<T>> {
    /// Flattens a tree of trees into a single tree.
    /// This is the `join` operation of a Monad.
    pub fn join(self) -> ConstTree<T> {
        // The root of the outer tree has a value which is the inner tree.
        let inner_tree = self.value().clone();

        // The new children are the children of the inner tree,
        // plus the result of recursively joining the children of the outer tree.
        let mut new_children = inner_tree.children().to_vec();
        let joined_outer_children = self.children().iter().map(|c| c.clone().join());
        new_children.extend(joined_outer_children);

        ConstTree::with_children(inner_tree.value().clone(), new_children)
    }
}

impl<T: Clone> ConstTree<T> {
    /// Consumes the tree and creates a new tree by applying a function to each value.
    ///
    /// This is the consuming equivalent of `map`, analogous to `into_iter`.
    /// This method is designed for compatibility with generic, consuming HKT traits.
    pub fn into_map<F, U>(self, mut f: F) -> ConstTree<U>
    where
        F: FnMut(T) -> U,
        U: Clone,
    {
        // A recursive helper function to handle the consumption.
        fn recursive_into_map<T, U, F>(tree: ConstTree<T>, f: &mut F) -> ConstTree<U>
        where
            T: Clone,
            F: FnMut(T) -> U,
            U: Clone,
        {
            // Try to unwrap the Arc to take ownership of the Node.
            // If it fails, it means the node is shared, so we must clone its contents.
            let node = Arc::try_unwrap(tree.node).unwrap_or_else(|arc| (*arc).clone());

            let new_value = f(node.value);
            let new_children: Vec<_> = node
                .children
                .into_iter()
                .map(|c| recursive_into_map(c, f))
                .collect();
            ConstTree::with_children(new_value, new_children)
        }

        recursive_into_map(self, &mut f)
    }
}

impl<T> ConstTree<T> {
    /// Recursively creates a new tree by applying a function to each value.
    /// This is the functional `map` operation, applied in pre-order.
    pub fn map<F, U>(&self, f: &mut F) -> ConstTree<U>
    where
        F: FnMut(&T) -> U,
        U: Clone, // The new value type must be cloneable
    {
        let new_value = f(self.value());
        let new_children: Vec<_> = self.children().iter().map(|child| child.map(f)).collect();
        ConstTree::with_children(new_value, new_children)
    }
}
