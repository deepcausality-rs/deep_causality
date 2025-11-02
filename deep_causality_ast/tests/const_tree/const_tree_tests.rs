#![allow(dead_code)]
#![allow(unused_imports)]

use deep_causality_ast::ConstTree;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_new_leaf() {
    let leaf = ConstTree::new(10);
    assert_eq!(*leaf.value(), 10);
    assert!(leaf.is_leaf());
    assert_eq!(leaf.children().len(), 0);
}

#[test]
fn test_with_children() {
    let leaf1 = ConstTree::new(1);
    let leaf2 = ConstTree::new(2);
    let tree = ConstTree::with_children(0, vec![leaf1.clone(), leaf2.clone()]);

    assert_eq!(*tree.value(), 0);
    assert!(!tree.is_leaf());
    assert_eq!(tree.children().len(), 2);
    assert_eq!(tree.children()[0], leaf1);
    assert_eq!(tree.children()[1], leaf2);
}

#[test]
fn test_get_child() {
    let leaf1 = ConstTree::new(1);
    let leaf2 = ConstTree::new(2);
    let tree = ConstTree::with_children(0, vec![leaf1.clone(), leaf2.clone()]);

    assert_eq!(tree.get_child(0), Some(&leaf1));
    assert_eq!(tree.get_child(1), Some(&leaf2));
    assert_eq!(tree.get_child(2), None);
}

#[test]
fn test_size() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::new(2),
            ConstTree::with_children(3, vec![ConstTree::new(4)]),
        ],
    );
    assert_eq!(tree.size(), 4);
    assert_eq!(ConstTree::new(0).size(), 1);
}

#[test]
fn test_depth() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::new(2),
            ConstTree::with_children(3, vec![ConstTree::new(4)]),
        ],
    );
    assert_eq!(tree.depth(), 3);
    let leaf = ConstTree::new(0);
    assert_eq!(leaf.depth(), 1);
    let empty_children_tree: ConstTree<i32> = ConstTree::with_children(0, vec![]);
    assert_eq!(empty_children_tree.depth(), 1);
}

#[test]
fn test_modification_methods() {
    let original = ConstTree::with_children(10, vec![ConstTree::new(11)]);

    // with_value
    let modified_value = original.with_value(20);
    assert_eq!(*original.value(), 10);
    assert_eq!(*modified_value.value(), 20);
    assert_eq!(original.children(), modified_value.children()); // Children are shared

    // add_child
    let added_child = original.add_child(ConstTree::new(12));
    assert_eq!(original.children().len(), 1);
    assert_eq!(added_child.children().len(), 2);
    assert_eq!(*added_child.children()[1].value(), 12);

    // replace_children
    let replaced_children = original.replace_children(vec![ConstTree::new(100)]);
    assert_eq!(original.children().len(), 1);
    assert_eq!(replaced_children.children().len(), 1);
    assert_ne!(original.children(), replaced_children.children());
    assert_eq!(*replaced_children.children()[0].value(), 100);

    // update_child
    let updated_child = original.update_child(0, ConstTree::new(111)).unwrap();
    assert_eq!(*original.children()[0].value(), 11);
    assert_eq!(*updated_child.children()[0].value(), 111);
    assert!(original.update_child(1, ConstTree::new(999)).is_none());

    // remove_child
    let removed_child = original.remove_child(0).unwrap();
    assert_eq!(original.children().len(), 1);
    assert!(removed_child.is_leaf());
    assert!(original.remove_child(1).is_none());
}

#[test]
fn test_map() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2), ConstTree::new(3)]);
    let mapped_tree = tree.map(&mut |v| v * 2);

    assert_eq!(*mapped_tree.value(), 2);
    assert_eq!(*mapped_tree.children()[0].value(), 4);
    assert_eq!(*mapped_tree.children()[1].value(), 6);
    // Original is unchanged
    assert_eq!(*tree.value(), 1);
}

#[test]
fn test_equality() {
    let tree1 = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let tree2 = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let tree3 = ConstTree::with_children(1, vec![ConstTree::new(99)]);
    let leaf = ConstTree::new(1);

    // Deep equality
    assert_eq!(tree1, tree2);
    assert_ne!(tree1, tree3);
    assert_ne!(tree1, leaf);

    // Pointer equality
    let tree1_clone = tree1.clone();
    assert_eq!(tree1, tree1_clone);
    // Check that the internal Arcs point to the same allocation
    assert!(tree1.ptr_eq(&tree1_clone));
    // tree2 has the same structure but is a different allocation
    assert!(!tree1.ptr_eq(&tree2));
}

#[test]
fn test_search() {
    let tree = ConstTree::with_children(
        10,
        vec![
            ConstTree::new(20),
            ConstTree::with_children(30, vec![ConstTree::new(40)]),
        ],
    );

    // find
    let found = tree.find(|v| *v == 30).unwrap();
    assert_eq!(*found.value(), 30);
    assert!(!found.is_leaf());

    assert!(tree.find(|v| *v == 99).is_none());

    // find_all
    let all_gt_15: Vec<_> = tree.find_all(|v| *v > 15).map(|n| *n.value()).collect();
    assert_eq!(all_gt_15, vec![20, 30, 40]);

    // contains
    assert!(tree.contains(&20));
    assert!(!tree.contains(&99));
}

#[test]
fn test_iterators() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );

    // Pre-order
    let pre_order_vals: Vec<_> = tree.iter_pre_order().copied().collect();
    assert_eq!(pre_order_vals, vec![1, 2, 3, 4]);

    // Post-order
    let post_order_vals: Vec<_> = tree.iter_post_order().copied().collect();
    assert_eq!(post_order_vals, vec![3, 2, 4, 1]);

    // Level-order
    let level_order_vals: Vec<_> = tree.iter_level_order().copied().collect();
    assert_eq!(level_order_vals, vec![1, 2, 4, 3]);
}

#[test]
fn test_thread_safety() {
    let tree = Arc::new(ConstTree::with_children(1, vec![ConstTree::new(2)]));
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let tree_clone = Arc::clone(&tree);
            thread::spawn(move || {
                // Each thread works with its clone
                let modified = tree_clone.add_child(ConstTree::new(100));
                assert_eq!(modified.children().len(), 2);
                assert_eq!(*modified.children()[1].value(), 100);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Original tree is unchanged
    assert_eq!(tree.children().len(), 1);
}

#[test]
fn test_display_and_debug() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2)]);

    // Display
    let display_str = format!("{}", tree);
    let expected_display = "1\n    2\n";
    assert_eq!(display_str, expected_display);

    // Debug
    let debug_str = format!("{:?}", tree);
    let expected_debug = "ConstTree { value: 1, children_count: 1 }";
    assert_eq!(debug_str, expected_debug);
}

#[test]
fn test_default() {
    let tree: ConstTree<i32> = ConstTree::default();
    assert_eq!(*tree.value(), 0);
    assert!(tree.is_leaf());
}

#[test]
fn test_from() {
    let tree = ConstTree::from(42);
    assert_eq!(*tree.value(), 42);
    assert!(tree.is_leaf());
}

#[test]
fn test_consuming_iterator() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2), ConstTree::new(3)]);
    let tree_clone = tree.clone(); // Clone to prove original is moved.

    // into_iter consumes the tree.
    let values: Vec<_> = tree.into_iter().collect();
    assert_eq!(values, vec![1, 2, 3]);

    // The original `tree` variable is now moved and cannot be used.
    // assert_eq!(*tree.value(), 1); // This line would fail to compile.

    // The clone is still valid.
    assert_eq!(*tree_clone.value(), 1);
}

#[test]
fn test_node_iterator() {
    let child1 = ConstTree::new(2);
    let child2 = ConstTree::new(3);
    let tree = ConstTree::with_children(1, vec![child1.clone(), child2.clone()]);

    let nodes: Vec<_> = tree.iter_nodes_pre_order().collect();

    assert_eq!(nodes.len(), 3);
    assert!(nodes[0].ptr_eq(&tree));
    assert!(nodes[1].ptr_eq(&child1));
    assert!(nodes[2].ptr_eq(&child2));
}

#[test]
fn test_into_map() {
    let tree = ConstTree::with_children(10, vec![ConstTree::new(20)]);
    let tree_clone = tree.clone();

    // into_map consumes the tree.
    let mapped_tree = tree.into_map(|v| v.to_string());

    assert_eq!(*mapped_tree.value(), "10");
    assert_eq!(*mapped_tree.children()[0].value(), "20");

    // Original `tree` is moved.
    // The clone is still valid and unchanged.
    assert_eq!(*tree_clone.value(), 10);
    assert_eq!(*tree_clone.children()[0].value(), 20);
}

#[test]
fn test_consuming_iterator_shared_arc() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2), ConstTree::new(3)]);
    let tree_clone1 = tree.clone();
    let tree_clone2 = tree.clone(); // Ensure multiple references

    // Call into_iter on one of the clones. This should trigger the Err branch.
    let values: Vec<_> = tree_clone1.into_iter().collect();
    assert_eq!(values, vec![1, 2, 3]);

    // Verify that the other clone is still valid.
    assert_eq!(*tree_clone2.value(), 1);
    assert_eq!(tree_clone2.children().len(), 2);
}

#[test]
fn test_join() {
    // Create a tree of trees: ConstTree<ConstTree<i32>>
    // Structure:
    //   Inner1(1)
    //   - Inner2(2)
    //     - Leaf(3)
    //   - Leaf(4)
    let leaf3 = ConstTree::new(3);
    let inner2 = ConstTree::with_children(2, vec![leaf3]);
    let leaf4 = ConstTree::new(4);
    let inner1 = ConstTree::with_children(1, vec![inner2, leaf4]);

    // Wrap them in an outer tree
    let tree_of_trees = ConstTree::new(inner1);

    // Join the tree
    let joined_tree = tree_of_trees.join();

    // Expected structure after join:
    //   1
    //   - 2
    //     - 3
    //   - 4
    assert_eq!(*joined_tree.value(), 1);
    assert_eq!(joined_tree.children().len(), 2);
    assert_eq!(*joined_tree.children()[0].value(), 2);
    assert_eq!(*joined_tree.children()[1].value(), 4);
    assert_eq!(joined_tree.children()[0].children().len(), 1);
    assert_eq!(*joined_tree.children()[0].children()[0].value(), 3);
    assert!(joined_tree.children()[1].is_leaf());

    let expected_vals: Vec<_> = joined_tree.iter_pre_order().copied().collect();
    assert_eq!(expected_vals, vec![1, 2, 3, 4]);
}
