/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_modification_methods() {
    let original = ConstTree::with_children(10, vec![ConstTree::new(11)]);

    // with_value (this is actually in mod.rs, but tested here for convenience with other modification methods)
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
