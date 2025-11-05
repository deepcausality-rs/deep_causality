### 1. The Core Data Structures

The API will expose a single public type, `ConstTree<T>`, which is an inexpensive, cloneable handle to the root of a tree. The internal `Node` structure is kept private to enforce the API's guarantees.

```rust
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;

// The internal, private representation of a single node in the tree.
// Deriving Debug is useful for internal debugging.
#[derive(Debug)]
struct Node<T> {
    value: T,
    children: Vec<ConstTree<T>>, // Children are also trees. This makes the structure recursive.
}

/// A persistent, immutable tree structure.
///
/// `ConstTree` is a handle to a tree node. Cloning a `ConstTree` is a cheap,
/// constant-time operation as it only increments a reference count.
///
/// All modification methods (`with_value`, `add_child`, etc.) are non-destructive.
/// They return a new `ConstTree` representing the modified version, leaving the
/// original unchanged and sharing as much memory as possible.
pub struct ConstTree<T> {
    // The public API wraps an Arc around the private Node.
    // This is the core of the persistent data structure pattern.
    node: Arc<Node<T>>,
}
```

### 2. Constructors: Creating a Tree

These methods provide ways to build a tree from scratch.

```rust
impl<T> ConstTree<T> {
    /// Creates a new `ConstTree` with a single root node and no children (a leaf).
    ///
    /// # Example
    /// ```
    /// let leaf = ConstTree::new(10);
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            node: Arc::new(Node {
                value,
                children: Vec::new(),
            }),
        }
    }

    /// Creates a new `ConstTree` with a root node and a given set of children.
    ///
    /// The `children` argument can be any type that can be converted into an iterator
    /// over `ConstTree<T>`, such as a `Vec<ConstTree<T>>` or a slice `&[ConstTree<T>]`.
    ///
    /// # Example
    /// ```
    /// let leaf1 = ConstTree::new(1);
    /// let leaf2 = ConstTree::new(2);
    /// let tree = ConstTree::with_children(0, vec![leaf1, leaf2]);
    /// ```
    pub fn with_children(value: T, children: impl IntoIterator<Item = Self>) -> Self {
        Self {
            node: Arc::new(Node {
                value,
                children: children.into_iter().collect(),
            }),
        }
    }
}
```

### 3. Accessors and Inspection

These methods allow you to inspect the tree without modifying it.

```rust
impl<T> ConstTree<T> {
    /// Returns a reference to the value stored at the root of this tree.
    pub fn value(&self) -> &T {
        &self.node.value
    }

    /// Returns a slice containing the children of the root of this tree.
    pub fn children(&self) -> &[ConstTree<T>] {
        &self.node.children
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

        if self.node.value != () { // Assuming a default value check might be needed
            queue.push_back((self, 1));
        }

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
        self.iter_nodes_pre_order().find(|node| predicate(node.value()))
    }

    /// Returns an iterator over all nodes that satisfy a predicate in pre-order.
    pub fn find_all<P>(&self, predicate: P) -> impl Iterator<Item = &ConstTree<T>>
    where
        P: Fn(&T) -> bool,
    {
        self.iter_nodes_pre_order().filter(move |node| predicate(node.value()))
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
```

### 4. Modification Methods: The "Persistent" API

This is the heart of the design. Each method takes `&self` and returns a new `Self`, creating a new version of the tree. Trait bounds are applied granularly.

```rust
impl<T> ConstTree<T> {
    /// Returns a new `ConstTree` with the root value replaced.
    /// The children of the new tree are shared with the original tree.
    pub fn with_value(&self, new_value: T) -> Self {
        Self {
            node: Arc::new(Node {
                value: new_value,
                children: self.node.children.clone(),
            }),
        }
    }

    /// Recursively creates a new tree by applying a function to each value.
    /// This is the functional `map` operation, applied in pre-order.
    pub fn map<F, U>(&self, f: &mut F) -> ConstTree<U>
    where
        F: FnMut(&T) -> U,
        U: Clone, // The new value type must be cloneable
    {
        let new_value = f(self.value());
        let new_children = self.children().iter().map(|child| child.map(f)).collect();
        ConstTree::with_children(new_value, new_children)
    }
}

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
```

### 5. Iteration

A good API should integrate with Rust's iterator ecosystem. We provide pre-order, post-order, and level-order iterators.

```rust
impl<T> ConstTree<T> {
    /// Returns an iterator that traverses the tree's values in pre-order (root, then children).
    pub fn iter_pre_order(&self) -> PreOrderIter<'_, T> {
        PreOrderIter { stack: vec![self] }
    }

    /// Returns an iterator that traverses the tree's nodes in pre-order.
    pub fn iter_nodes_pre_order(&self) -> PreOrderNodeIter<'_, T> {
        PreOrderNodeIter { stack: vec![self] }
    }

    /// Returns an iterator that traverses the tree's values in post-order (children, then root).
    pub fn iter_post_order(&self) -> PostOrderIter<'_, T> {
        PostOrderIter::new(self)
    }

    /// Returns an iterator that traverses the tree's values in level-order (breadth-first).
    pub fn iter_level_order(&self) -> LevelOrderIter<'_, T> {
        let mut queue = VecDeque::new();
        queue.push_back(self);
        LevelOrderIter { queue }
    }
}

/// An iterator that traverses a `ConstTree` in pre-order (root, then children).
pub struct PreOrderIter<'a, T> {
    stack: Vec<&'a ConstTree<T>>,
}

impl<'a, T> Iterator for PreOrderIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let tree = self.stack.pop()?;
        self.stack.extend(tree.children().iter().rev());
        Some(tree.value())
    }
}

/// An iterator that traverses a `ConstTree`'s nodes in pre-order.
pub struct PreOrderNodeIter<'a, T> {
    stack: Vec<&'a ConstTree<T>>,
}

impl<'a, T> Iterator for PreOrderNodeIter<'a, T> {
    type Item = &'a ConstTree<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let tree = self.stack.pop()?;
        self.stack.extend(tree.children().iter().rev());
        Some(tree)
    }
}

/// An iterator that traverses a `ConstTree` in post-order (children, then root).
/// It maintains a stack containing tuples of a node and an iterator over its children.
pub struct PostOrderIter<'a, T> {
    stack: Vec<(&'a ConstTree<T>, std::slice::Iter<'a, ConstTree<T>>) >,
}

impl<'a, T> PostOrderIter<'a, T> {
    /// Creates a new post-order iterator starting at the given root.
    pub fn new(root: &'a ConstTree<T>) -> Self {
        let mut iter = PostOrderIter { stack: Vec::new() };
        let children_iter = root.children().iter();
        iter.stack.push((root, children_iter));
        iter
    }
}

impl<'a, T> Iterator for PostOrderIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // Loop until we find the next node to yield or the stack is empty.
        loop {
            // Peek at the top of the stack to see which node we're processing.
            let (current_node, children_iter) = match self.stack.last_mut() {
                Some(top) => top,
                None => return None, // Nothing left on the stack, traversal is complete.
            };

            // Try to get the next child of the current node.
            match children_iter.next() {
                Some(child_node) => {
                    // If there is a child, create its own entry and push it onto the stack.
                    let grand_children_iter = child_node.children().iter();
                    self.stack.push((child_node, grand_children_iter));
                }
                None => {
                    // If there are no more children, it means we have visited all descendants
                    // of `current_node`. It is now time to visit `current_node` itself.
                    // We pop it from the stack and return its value.
                    let (finished_node, _) = self.stack.pop().unwrap();
                    return Some(finished_node.value());
                }
            }
        }
    }
}


/// An iterator that traverses a `ConstTree` in level-order (breadth-first).
pub struct LevelOrderIter<'a, T> {
    queue: VecDeque<&'a ConstTree<T>>,
}

impl<'a, T> Iterator for LevelOrderIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let tree = self.queue.pop_front()?;
        self.queue.extend(tree.children().iter());
        Some(tree.value())
    }
}
```

### 6. Trait Implementations & Ergonomics

Standard traits make the `ConstTree` easy to use.

```rust
impl<T> ConstTree<T> {
    // Private helper for Display trait to recursively print the tree with indentation.
    fn fmt_recursive(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result where T: fmt::Display {
        writeln!(f, "{:indent$}{}", "", self.node.value, indent = indent * 4)?; // 4 spaces per indent level
        for child in &self.node.children {
            child.fmt_recursive(f, indent + 1)?;
        }
        Ok(())
    }
}

// Clone is cheap because it just clones the Arc.
impl<T> Clone for ConstTree<T> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
        }
    }
}

// Allow comparing trees for equality if their values can be compared.
impl<T: PartialEq> PartialEq for ConstTree<T> {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.node, &other.node) {
            return true;
        }
        self.node.value == other.node.value && self.node.children == other.node.children
    }
}
impl<T: Eq> Eq for ConstTree<T> {}

// Provide a useful debug representation.
impl<T: fmt::Debug> fmt::Debug for ConstTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConstTree")
         .field("value", &self.node.value)
         .field("children_count", &self.node.children.len())
         .finish()
    }
}

// Provide a human-readable string representation of the tree.
impl<T: fmt::Display> fmt::Display for ConstTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_recursive(f, 0)
    }
}

// Create a default tree if the value type supports it.
impl<T: Default> Default for ConstTree<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

// Conveniently create a leaf node from a value.
impl<T> From<T> for ConstTree<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
```

### 7. Thread Safety

The `ConstTree<T>` is designed to be thread-safe.

Because `ConstTree<T>` is built on `std::sync::Arc`, it automatically and conditionally implements the `Send` and `Sync` marker traits.

*   `ConstTree<T>` is `Send` if `T` is `Send + Sync`.
*   `ConstTree<T>` is `Sync` if `T` is `Send + Sync`.

This means you can safely share `ConstTree<T>` instances across threads as long as the data `T` stored within the tree is itself thread-safe. No special handling is required; the Rust compiler enforces this automatically.

### 8. Testing Strategy

A robust testing strategy is crucial.

1.  **Doctests:** All public methods should have clear examples in their documentation comments. These serve as both examples and tests that are run with `cargo test`.
2.  **Unit & Integration Tests:** A dedicated test suite in the `/tests` directory should be created to cover more complex scenarios:
    *   **Equality:** Test both pointer equality (`Arc::ptr_eq`) and deep structural equality.
    *   **Modification:** Verify that original trees are unchanged after modification operations.
    *   **Iteration:** Test all iterator types (`PreOrder`, `PostOrder`, `LevelOrder`) on various tree shapes (empty, leaf, complex).
    *   **Search:** Test `find`, `find_all`, and `contains` for various scenarios (found, not found, multiple matches).
    *   **Thread Safety:** Write tests that send `ConstTree` instances across threads to verify `Send` and `Sync` compliance.
    *   **Edge Cases:** Test behavior with empty children, deep recursion (to a reasonable limit), and different data types for `T`.
