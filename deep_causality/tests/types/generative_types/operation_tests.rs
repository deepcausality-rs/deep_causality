/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseSymbol, Data, EuclideanSpace, EuclideanSpacetime, EuclideanTime, Operation,
};
use deep_causality_ast::ConstTree;

// Type aliases for testing
type TestData = Data<f64>;
type TestSpace = EuclideanSpace;
type TestTime = EuclideanTime;
type TestSpacetime = EuclideanSpacetime;
type TestSymbol = BaseSymbol;

#[test]
fn test_operation_create_context() {
    let op = Operation::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::CreateContext {
        id: 1,
        name: "test_context".to_string(),
        capacity: 10,
    };

    match op {
        Operation::CreateContext { id, name, capacity } => {
            assert_eq!(id, 1);
            assert_eq!(name, "test_context");
            assert_eq!(capacity, 10);
        }
        _ => panic!("Wrong operation variant"),
    }
}

#[test]
fn test_operation_create_extra_context() {
    let op = Operation::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::CreateExtraContext {
        context_id: 1,
        extra_context_id: 2,
        capacity: 5,
    };

    match op {
        Operation::CreateExtraContext {
            context_id,
            extra_context_id,
            capacity,
        } => {
            assert_eq!(context_id, 1);
            assert_eq!(extra_context_id, 2);
            assert_eq!(capacity, 5);
        }
        _ => panic!("Wrong operation variant"),
    }
}

#[test]
fn test_operation_update_context() {
    let op = Operation::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::UpdateContext {
        id: 1,
        new_name: Some("updated_name".to_string()),
    };

    match op {
        Operation::UpdateContext { id, new_name } => {
            assert_eq!(id, 1);
            assert_eq!(new_name, Some("updated_name".to_string()));
        }
        _ => panic!("Wrong operation variant"),
    }
}

#[test]
fn test_operation_update_context_no_name() {
    let op = Operation::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::UpdateContext {
        id: 1,
        new_name: None,
    };

    match op {
        Operation::UpdateContext { id, new_name } => {
            assert_eq!(id, 1);
            assert!(new_name.is_none());
        }
        _ => panic!("Wrong operation variant"),
    }
}

#[test]
fn test_operation_delete_context() {
    let op = Operation::<(), (), TestData, TestSpace, TestTime, TestSpacetime, TestSymbol, f64, f64>::DeleteContext(42);

    match op {
        Operation::DeleteContext(id) => {
            assert_eq!(id, 42);
        }
        _ => panic!("Wrong operation variant"),
    }
}

#[test]
fn test_operation_delete_causaloid() {
    let op = Operation::<(), (), TestData, TestSpace, TestTime, TestSpacetime, TestSymbol, f64, f64>::DeleteCausaloid(99);

    match op {
        Operation::DeleteCausaloid(id) => {
            assert_eq!(id, 99);
        }
        _ => panic!("Wrong operation variant"),
    }
}

#[test]
fn test_operation_clone() {
    let op1 = Operation::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::CreateContext {
        id: 1,
        name: "test".to_string(),
        capacity: 10,
    };

    let op2 = op1.clone();

    match (op1, op2) {
        (
            Operation::CreateContext {
                id: id1,
                name: name1,
                capacity: cap1,
            },
            Operation::CreateContext {
                id: id2,
                name: name2,
                capacity: cap2,
            },
        ) => {
            assert_eq!(id1, id2);
            assert_eq!(name1, name2);
            assert_eq!(cap1, cap2);
        }
        _ => panic!("Clone failed"),
    }
}

#[test]
fn test_op_tree_single_node() {
    let op = Operation::<(), (), TestData, TestSpace, TestTime, TestSpacetime, TestSymbol, f64, f64>::DeleteContext(1);
    let tree: ConstTree<
        Operation<(), (), TestData, TestSpace, TestTime, TestSpacetime, TestSymbol, f64, f64>,
    > = ConstTree::new(op);

    assert_eq!(tree.children().len(), 0);
}

#[test]
fn test_op_tree_with_children() {
    let root_op = Operation::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::CreateContext {
        id: 1,
        name: "root".to_string(),
        capacity: 10,
    };

    let child_op1 = Operation::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::CreateContext {
        id: 2,
        name: "child1".to_string(),
        capacity: 5,
    };

    let child_op2 = Operation::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::DeleteContext(3);

    let child_tree1 = ConstTree::new(child_op1);
    let child_tree2 = ConstTree::new(child_op2);

    let tree = ConstTree::with_children(root_op, vec![child_tree1, child_tree2]);

    assert_eq!(tree.children().len(), 2);
}

#[test]
fn test_operation_debug_format() {
    let op = Operation::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    >::CreateContext {
        id: 1,
        name: "test".to_string(),
        capacity: 10,
    };

    let debug_str = format!("{:?}", op);
    assert!(debug_str.contains("CreateContext"));
}
