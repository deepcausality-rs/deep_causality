/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_map() {
    // Three levels. On a depth-1 tree the recursive step is only ever entered on leaves, so a
    // `map` that rebuilt each child as a childless node — silently dropping every grandchild —
    // passes.
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(4), ConstTree::new(5)]),
            ConstTree::new(3),
        ],
    );
    let mapped_tree = tree.map(&mut |v| v * 2);

    assert_eq!(
        mapped_tree.iter_pre_order().copied().collect::<Vec<_>>(),
        vec![2, 4, 8, 10, 6]
    );
    assert_eq!(mapped_tree.size(), tree.size());
    assert_eq!(mapped_tree.depth(), tree.depth());
    // Original is unchanged
    assert_eq!(
        tree.iter_pre_order().copied().collect::<Vec<_>>(),
        vec![1, 2, 4, 5, 3]
    );
}

#[test]
fn test_map_visits_every_node_root_first() {
    // A stateful closure records the visit order, pinning that `f` is applied to the root before
    // its children and left to right across siblings.
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );
    let mut seen = Vec::new();
    let mapped = tree.map(&mut |v| {
        seen.push(*v);
        *v
    });
    assert_eq!(seen, vec![1, 2, 3, 4]);
    assert_eq!(mapped, tree);
}

#[test]
fn test_into_map() {
    // Same depth argument as `map`, plus two children so left-to-right order is observable.
    let tree = ConstTree::with_children(
        10,
        vec![
            ConstTree::with_children(20, vec![ConstTree::new(30)]),
            ConstTree::new(40),
        ],
    );
    let tree_clone = tree.clone();

    // into_map consumes the tree.
    let mapped_tree = tree.into_map(|v| v.to_string());

    assert_eq!(
        mapped_tree.iter_pre_order().cloned().collect::<Vec<_>>(),
        vec!["10", "20", "30", "40"]
    );

    // Original `tree` is moved. The clone is still valid and unchanged.
    assert_eq!(
        tree_clone.iter_pre_order().copied().collect::<Vec<_>>(),
        vec![10, 20, 30, 40]
    );
}

#[test]
fn test_into_map_on_an_unshared_tree_takes_the_owned_path() {
    // No clone outlives the call, so `Arc::try_unwrap` succeeds and the owning arm runs. The
    // other arm is covered by `test_into_map`, which holds a clone.
    let tree = ConstTree::with_children(
        1,
        vec![ConstTree::with_children(2, vec![ConstTree::new(3)])],
    );
    let mapped = tree.into_map(|v| v * 10);
    assert_eq!(
        mapped.iter_pre_order().copied().collect::<Vec<_>>(),
        vec![10, 20, 30]
    );
}

/// A tree of trees whose outer root and outer children both carry inner trees:
///
/// ```text
/// outer:  [1 -> 2]  ->  [ [5 -> 6] -> [ [8] ],  [7] ]
/// joined: 1 -> [ 2, 5 -> [6, 8], 7 ]
/// ```
fn tree_of_trees() -> ConstTree<ConstTree<i32>> {
    let inner_root = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let inner_b = ConstTree::with_children(5, vec![ConstTree::new(6)]);
    ConstTree::with_children(
        inner_root,
        vec![
            ConstTree::with_children(inner_b, vec![ConstTree::new(ConstTree::new(8))]),
            ConstTree::new(ConstTree::new(7)),
        ],
    )
}

#[test]
fn test_join() {
    // The outer root used to be a leaf, so the one line of `join` that actually flattens — the
    // one extending the inner root's children with the joined outer children — never ran, and
    // an implementation that discarded every outer subtree passed.
    let joined = tree_of_trees().join();

    assert_eq!(*joined.value(), 1);
    // The inner root's own child first, then one flattened subtree per outer child, in order.
    assert_eq!(joined.children().len(), 3);
    assert_eq!(*joined.children()[0].value(), 2);
    assert_eq!(*joined.children()[1].value(), 5);
    assert_eq!(*joined.children()[2].value(), 7);
    assert!(joined.children()[0].is_leaf());
    assert!(joined.children()[2].is_leaf());

    // `join` recurses into the outer children, so the grandchild's inner tree lands beside the
    // inner tree's own child rather than being dropped or nested a level too deep.
    assert_eq!(joined.children()[1].children().len(), 2);
    assert_eq!(*joined.children()[1].children()[0].value(), 6);
    assert_eq!(*joined.children()[1].children()[1].value(), 8);

    assert_eq!(
        joined.iter_pre_order().copied().collect::<Vec<_>>(),
        vec![1, 2, 5, 6, 8, 7]
    );
}

#[test]
fn test_join_on_a_shared_tree_clones_instead_of_unwrapping() {
    // `Arc::try_unwrap` fails while another handle is alive, taking the `unwrap_or_else` clone
    // arm. Nothing exercised it: the other join fixture is never cloned, so only the owning
    // path ran. Both arms must produce the same tree.
    let shared = tree_of_trees();
    let keep_alive = shared.clone();

    let joined = shared.join();
    assert_eq!(
        joined.iter_pre_order().copied().collect::<Vec<_>>(),
        vec![1, 2, 5, 6, 8, 7]
    );
    // The surviving handle is untouched by the join.
    assert_eq!(*keep_alive.value().value(), 1);
    assert_eq!(keep_alive.children().len(), 2);
}
