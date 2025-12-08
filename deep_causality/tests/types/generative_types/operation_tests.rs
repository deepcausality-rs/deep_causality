/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseContext, BaseContextoid, Causaloid, ContextoidType, Data, Identifiable, OpTree, Operation,
};
use std::sync::{Arc, RwLock};

// Type aliases for testing to reduce verbosity
type TestInput = i32;
type TestOutput = i32;
// Using BaseContext and BaseContextoid as suggested
type TestContext = BaseContext;
type TestNode = BaseContextoid;

// Helper function for Causaloid. Must be a fn pointer, not a closure.
fn dummy_causal_fn(_: TestInput) -> deep_causality::PropagatingEffect<TestOutput> {
    deep_causality::PropagatingEffect::from_value(1)
}

// Helper to create a dummy Causaloid
fn create_dummy_causaloid(
    id: u64,
) -> Causaloid<TestInput, TestOutput, (), Arc<RwLock<TestContext>>> {
    Causaloid::new(id, dummy_causal_fn, "test_causaloid")
}

// Helper to create a dummy Contextoid
fn create_dummy_contextoid(id: u64) -> TestNode {
    TestNode::new(id, ContextoidType::Datoid(Data::new(id, 0.0)))
}

#[test]
fn test_causaloid_operations() {
    let id = 1;
    let causaloid = create_dummy_causaloid(id);

    // Test CreateCausaloid
    let create_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::CreateCausaloid(id, causaloid.clone());

    match &create_op {
        Operation::CreateCausaloid(cid, c) => {
            assert_eq!(*cid, id);
            assert_eq!(c.id(), id);
        }
        _ => panic!("Expected CreateCausaloid"),
    }

    // Test UpdateCausaloid
    let update_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::UpdateCausaloid(id, causaloid.clone());

    match &update_op {
        Operation::UpdateCausaloid(cid, c) => {
            assert_eq!(*cid, id);
            assert_eq!(c.id(), id);
        }
        _ => panic!("Expected UpdateCausaloid"),
    }

    // Test DeleteCausaloid
    let delete_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::DeleteCausaloid(id);

    match &delete_op {
        Operation::DeleteCausaloid(cid) => {
            assert_eq!(*cid, id);
        }
        _ => panic!("Expected DeleteCausaloid"),
    }
}

#[test]
fn test_context_operations() {
    let ctx_id = 100;
    let name = "MainContext".to_string();
    let capacity = 10;

    // Test CreateContext
    let create_ctx_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::CreateContext {
            id: ctx_id,
            name: name.clone(),
            capacity,
        };

    match &create_ctx_op {
        Operation::CreateContext {
            id,
            name: n,
            capacity: c,
        } => {
            assert_eq!(*id, ctx_id);
            assert_eq!(n, &name);
            assert_eq!(*c, capacity);
        }
        _ => panic!("Expected CreateContext"),
    }

    // Test CreateExtraContext
    let extra_id = 101;
    let extra_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::CreateExtraContext {
            context_id: ctx_id,
            extra_context_id: extra_id,
            capacity,
        };

    match &extra_op {
        Operation::CreateExtraContext {
            context_id,
            extra_context_id,
            capacity: c,
        } => {
            assert_eq!(*context_id, ctx_id);
            assert_eq!(*extra_context_id, extra_id);
            assert_eq!(*c, capacity);
        }
        _ => panic!("Expected CreateExtraContext"),
    }

    // Test UpdateContext
    let new_name = "UpdatedContext".to_string();
    let update_ctx_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::UpdateContext {
            id: ctx_id,
            new_name: Some(new_name.clone()),
        };

    match &update_ctx_op {
        Operation::UpdateContext { id, new_name: n } => {
            assert_eq!(*id, ctx_id);
            assert_eq!(n.as_ref().unwrap(), &new_name);
        }
        _ => panic!("Expected UpdateContext"),
    }

    // Test DeleteContext
    let delete_ctx_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::DeleteContext(ctx_id);

    match &delete_ctx_op {
        Operation::DeleteContext(id) => {
            assert_eq!(*id, ctx_id);
        }
        _ => panic!("Expected DeleteContext"),
    }
}

#[test]
fn test_contextoid_operations() {
    let ctx_id = 200;
    let node_id = 1;

    let node_data = create_dummy_contextoid(node_id);
    let new_node_data = create_dummy_contextoid(node_id); // Same ID, but implicitly "new" instance

    // Test AddContextoidToContext
    let add_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::AddContextoidToContext {
            context_id: ctx_id,
            contextoid: node_data.clone(),
        };

    match &add_op {
        Operation::AddContextoidToContext {
            context_id,
            contextoid,
        } => {
            assert_eq!(*context_id, ctx_id);
            assert_eq!(contextoid.id(), node_id);
        }
        _ => panic!("Expected AddContextoidToContext"),
    }

    // Test UpdateContextoidInContext
    let update_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::UpdateContextoidInContext {
            context_id: ctx_id,
            existing_contextoid: node_id,
            new_contextoid: new_node_data.clone(),
        };

    match &update_op {
        Operation::UpdateContextoidInContext {
            context_id,
            existing_contextoid,
            new_contextoid,
        } => {
            assert_eq!(*context_id, ctx_id);
            assert_eq!(*existing_contextoid, node_id);
            assert_eq!(new_contextoid.id(), node_id);
        }
        _ => panic!("Expected UpdateContextoidInContext"),
    }

    // Test DeleteContextoidFromContext
    let delete_op: Operation<TestInput, TestOutput, TestContext, TestNode> =
        Operation::DeleteContextoidFromContext {
            context_id: ctx_id,
            contextoid_id: node_id,
        };

    match &delete_op {
        Operation::DeleteContextoidFromContext {
            context_id,
            contextoid_id,
        } => {
            assert_eq!(*context_id, ctx_id);
            assert_eq!(*contextoid_id, node_id);
        }
        _ => panic!("Expected DeleteContextoidFromContext"),
    }
}

#[test]
fn test_control_flow_operations() {
    // Test Sequence
    let seq_op: Operation<TestInput, TestOutput, TestContext, TestNode> = Operation::Sequence;
    match seq_op {
        Operation::Sequence => (),
        _ => panic!("Expected Sequence"),
    }

    // Test NoOp
    let no_op: Operation<TestInput, TestOutput, TestContext, TestNode> = Operation::NoOp;
    match no_op {
        Operation::NoOp => (),
        _ => panic!("Expected NoOp"),
    }
}

#[test]
fn test_optree_structure() {
    let id = 1;
    let causaloid = create_dummy_causaloid(id);
    let create_op =
        Operation::<TestInput, TestOutput, TestContext, TestNode>::CreateCausaloid(id, causaloid);
    let no_op = Operation::<TestInput, TestOutput, TestContext, TestNode>::NoOp;

    // Build a simple tree: Sequence -> [CreateCausaloid, NoOp]
    let leaf1 = OpTree::new(create_op);
    let leaf2 = OpTree::new(no_op);

    // Using Sequence as the root operation value
    let mut root = OpTree::new(Operation::Sequence);

    // Construct tree using add_child instead of push
    root = root.add_child(leaf1);
    root = root.add_child(leaf2);

    // ConstTree::len(self) doesn't exist? Wait, usually AST nodes have ways to count.
    // However, I can check children().len()

    assert_eq!(root.children().len(), 2);
    assert!(matches!(root.value(), Operation::Sequence));

    // Check children
    let children = root.children();
    let child1 = &children[0];
    let child2 = &children[1];

    assert!(matches!(child1.value(), Operation::CreateCausaloid(..)));
    assert!(matches!(child2.value(), Operation::NoOp));
}

#[test]
fn test_debug_and_clone() {
    let op = Operation::<TestInput, TestOutput, TestContext, TestNode>::NoOp;

    // Test Clone
    let op_clone = op.clone();
    assert!(matches!(op_clone, Operation::NoOp));

    // Test Debug
    let debug_str = format!("{:?}", op);
    assert_eq!(debug_str, "NoOp");

    let ctx_op = Operation::<TestInput, TestOutput, TestContext, TestNode>::DeleteContext(123);
    let debug_ctx = format!("{:?}", ctx_op);
    assert!(debug_ctx.contains("DeleteContext"));
    assert!(debug_ctx.contains("123"));
}
