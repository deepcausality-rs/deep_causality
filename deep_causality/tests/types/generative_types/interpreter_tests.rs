/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseContext, BaseContextoid, CausalSystemState, Causaloid, ContextoidType, Data, Interpreter,
    ModelValidationError, OpTree, Operation,
};
use std::sync::{Arc, RwLock};

// Type aliases
type TestInput = i32;
type TestOutput = i32;
type TestContext = BaseContext;
type TestNode = BaseContextoid;
type TestState = CausalSystemState<TestInput, TestOutput, TestContext>;

// Helper function for Causaloid. Must be a fn pointer.
fn dummy_causal_fn(_: TestInput) -> deep_causality::PropagatingEffect<TestOutput> {
    deep_causality::PropagatingEffect::from_value(1)
}

// Helpers
fn create_dummy_causaloid(
    id: u64,
) -> Causaloid<TestInput, TestOutput, (), Arc<RwLock<TestContext>>> {
    Causaloid::new(id, dummy_causal_fn, "test_causaloid")
}

fn create_dummy_contextoid(id: u64) -> TestNode {
    TestNode::new(id, ContextoidType::Datoid(Data::new(id, 0.0)))
}

fn create_sys_state() -> TestState {
    CausalSystemState::new()
}

#[test]
fn test_create_causaloid() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let id = 1;
    let causaloid = create_dummy_causaloid(id);
    let op = Operation::CreateCausaloid(id, causaloid);
    let tree = OpTree::new(op);

    // Success
    let result = interpreter.execute(&tree, state.clone());
    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert!(!result.logs.is_empty());
    assert_eq!(result.value.as_ref().unwrap().causaloids.len(), 1);
    assert!(result.value.unwrap().causaloids.contains_key(&id));

    // Failure: Duplicate
    // First, execute successfully to get a state with the causaloid
    let state_with_c = interpreter.execute(&tree, state).value.unwrap();
    // Try to create the same causaloid again
    let result_fail = interpreter.execute(&tree, state_with_c);
    assert!(result_fail.error.is_some());
    match result_fail.error.unwrap() {
        ModelValidationError::DuplicateCausaloidID { id: err_id } => assert_eq!(err_id, id),
        _ => panic!("Expected DuplicateCausaloidID error"),
    }
}

#[test]
fn test_update_causaloid() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let id = 1;
    let causaloid = create_dummy_causaloid(id);

    // Add Causaloid first
    let create_tree = OpTree::new(Operation::CreateCausaloid(id, causaloid.clone()));
    let state_with_c = interpreter
        .execute(&create_tree, state.clone())
        .value
        .unwrap();

    // Success
    let update_op = Operation::UpdateCausaloid(id, causaloid);
    let update_tree = OpTree::new(update_op);
    let result = interpreter.execute(&update_tree, state_with_c);
    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert!(result.value.unwrap().causaloids.contains_key(&id));

    // Failure: Not Found
    let result_fail = interpreter.execute(&update_tree, state); // Empty state
    assert!(result_fail.error.is_some());
    match result_fail.error.unwrap() {
        ModelValidationError::UpdateNodeError { .. } => (),
        _ => panic!("Expected UpdateNodeError"),
    }
}

#[test]
fn test_delete_causaloid() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let id = 1;
    let causaloid = create_dummy_causaloid(id);

    // Add Causaloid first
    let create_tree = OpTree::new(Operation::CreateCausaloid(id, causaloid));
    let state_with_c = interpreter
        .execute(&create_tree, state.clone())
        .value
        .unwrap();

    // Success
    let delete_op = Operation::DeleteCausaloid(id);
    let delete_tree = OpTree::new(delete_op);
    let result = interpreter.execute(&delete_tree, state_with_c);
    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert!(result.value.unwrap().causaloids.is_empty());

    // Failure: Not Found
    let result_fail = interpreter.execute(&delete_tree, state); // Empty state
    assert!(result_fail.error.is_some());
    match result_fail.error.unwrap() {
        ModelValidationError::RemoveNodeError { .. } => (),
        _ => panic!("Expected RemoveNodeError"),
    }
}

#[test]
fn test_create_context() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let id = 100;
    let name = "TestContext".to_string();
    let capacity = 10;
    let op = Operation::CreateContext {
        id,
        name: name.clone(),
        capacity,
    };
    let tree = OpTree::new(op);

    // Success
    let result = interpreter.execute(&tree, state.clone());
    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert!(result.value.unwrap().contexts.contains_key(&id));

    // Failure: Duplicate
    let state_with_ctx = interpreter.execute(&tree, state).value.unwrap();
    let result_fail = interpreter.execute(&tree, state_with_ctx);
    assert!(result_fail.error.is_some());
    match result_fail.error.unwrap() {
        ModelValidationError::DuplicateContextId { id: err_id } => assert_eq!(err_id, id),
        _ => panic!("Expected DuplicateContextId error"),
    }
}

#[test]
fn test_create_extra_context() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let ctx_id = 100;
    let extra_id = 101;

    // Check parent exists
    let create_extra_op = Operation::CreateExtraContext {
        context_id: ctx_id,
        extra_context_id: extra_id,
        capacity: 10,
    };
    let tree = OpTree::new(create_extra_op.clone());

    // Failure: Parent Not Found
    let result_fail_parent = interpreter.execute(&tree, state.clone());
    assert!(result_fail_parent.error.is_some());
    match result_fail_parent.error.unwrap() {
        ModelValidationError::TargetContextNotFound { id } => assert_eq!(id, ctx_id),
        _ => panic!("Expected TargetContextNotFound"),
    }

    // Prepare state with parent context
    let create_ctx_op = Operation::CreateContext {
        id: ctx_id,
        name: "C1".into(),
        capacity: 10,
    };
    let state_with_parent = interpreter
        .execute(&OpTree::new(create_ctx_op), state)
        .value
        .unwrap();

    // Success
    let result = interpreter.execute(&tree, state_with_parent.clone());
    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert!(result.value.unwrap().contexts.contains_key(&extra_id));

    // Failure: Duplicate Context ID
    // Create extra context first
    let state_with_extra = interpreter.execute(&tree, state_with_parent).value.unwrap();
    // Try to create it again
    let result_fail_dup = interpreter.execute(&tree, state_with_extra);
    assert!(result_fail_dup.error.is_some());
    match result_fail_dup.error.unwrap() {
        ModelValidationError::DuplicateContextId { id } => assert_eq!(id, extra_id),
        _ => panic!("Expected DuplicateContextId"),
    }
}

#[test]
fn test_update_context() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let id = 100;

    // Add context first
    let create_tree = OpTree::new(Operation::CreateContext {
        id,
        name: "Old".into(),
        capacity: 10,
    });
    let state_with_ctx = interpreter
        .execute(&create_tree, state.clone())
        .value
        .unwrap();

    // Success
    let update_op = Operation::UpdateContext {
        id,
        new_name: Some("New".into()),
    };
    let update_tree = OpTree::new(update_op);
    let result = interpreter.execute(&update_tree, state_with_ctx);
    assert!(result.value.is_some());
    assert!(
        result
            .value
            .as_ref()
            .unwrap()
            .contexts
            .get(&id)
            .unwrap()
            .name()
            == "New"
    );

    // Failure: Not Found
    let result_fail = interpreter.execute(&update_tree, state);
    assert!(result_fail.error.is_some());
    match result_fail.error.unwrap() {
        ModelValidationError::TargetContextNotFound { .. } => (),
        _ => panic!("Expected TargetContextNotFound"),
    }
}

#[test]
fn test_delete_context() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let id = 100;

    // Add context first
    let create_tree = OpTree::new(Operation::CreateContext {
        id,
        name: "DeleteMe".into(),
        capacity: 10,
    });
    let state_with_ctx = interpreter
        .execute(&create_tree, state.clone())
        .value
        .unwrap();

    // Success
    let delete_op = Operation::DeleteContext(id);
    let delete_tree = OpTree::new(delete_op);
    let result = interpreter.execute(&delete_tree, state_with_ctx);
    assert!(result.value.is_some());
    assert!(!result.value.unwrap().contexts.contains_key(&id));

    // Failure: Not Found
    let result_fail = interpreter.execute(&delete_tree, state);
    assert!(result_fail.error.is_some());
    match result_fail.error.unwrap() {
        ModelValidationError::TargetContextNotFound { .. } => (),
        _ => panic!("Expected TargetContextNotFound"),
    }
}

#[test]
fn test_add_contextoid_to_context() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let ctx_id = 100;
    let node_id = 1;
    let node = create_dummy_contextoid(node_id);

    // Prepare state with context
    let create_ctx_tree = OpTree::new(Operation::CreateContext {
        id: ctx_id,
        name: "C1".into(),
        capacity: 10,
    });
    let state_with_ctx = interpreter
        .execute(&create_ctx_tree, state.clone())
        .value
        .unwrap();

    let op = Operation::AddContextoidToContext {
        context_id: ctx_id,
        contextoid: node.clone(),
    };
    let tree = OpTree::new(op);

    // Success
    let result = interpreter.execute(&tree, state_with_ctx.clone());
    assert!(result.value.is_some());
    assert!(
        result
            .value
            .as_ref()
            .unwrap()
            .contexts
            .get(&ctx_id)
            .unwrap()
            .get_node_index_by_id(node_id)
            .is_some()
    );

    // Failure: Context Not Found
    let result_fail_ctx = interpreter.execute(&tree, state);
    assert!(result_fail_ctx.error.is_some());
    match result_fail_ctx.error.unwrap() {
        ModelValidationError::TargetContextNotFound { id } => assert_eq!(id, ctx_id),
        _ => panic!("Expected TargetContextNotFound"),
    }

    // Failure: Duplicate Node
    let state_with_node = result.value.unwrap();
    let result_fail_dup = interpreter.execute(&tree, state_with_node);
    assert!(result_fail_dup.error.is_some());
    match result_fail_dup.error.unwrap() {
        ModelValidationError::AddContextoidError { .. } => (),
        _ => panic!("Expected AddContextoidError"),
    }
}

#[test]
fn test_update_contextoid_in_context() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let ctx_id = 100;
    let node_id = 1;
    let node = create_dummy_contextoid(node_id);
    let new_node = create_dummy_contextoid(node_id); // Same ID

    // Prepare state with context and node
    let create_ctx_op = Operation::CreateContext {
        id: ctx_id,
        name: "C1".into(),
        capacity: 10,
    };
    let add_node_op = Operation::AddContextoidToContext {
        context_id: ctx_id,
        contextoid: node,
    };

    // Sequence to build state
    let root = OpTree::new(Operation::Sequence)
        .add_child(OpTree::new(create_ctx_op))
        .add_child(OpTree::new(add_node_op));

    let state_prepared = interpreter.execute(&root, state.clone()).value.unwrap();

    let update_op = Operation::UpdateContextoidInContext {
        context_id: ctx_id,
        existing_contextoid: node_id,
        new_contextoid: new_node,
    };
    let update_tree = OpTree::new(update_op.clone());

    // Success
    let result = interpreter.execute(&update_tree, state_prepared);
    assert!(result.value.is_some());
    assert!(result.error.is_none());

    // Failure: Context Not Found
    let result_fail_ctx = interpreter.execute(&update_tree, state.clone());
    match result_fail_ctx.error.unwrap() {
        ModelValidationError::TargetContextNotFound { .. } => (),
        _ => panic!("Expected TargetContextNotFound"),
    }

    // Failure: Node Not Found (Context exists but empty)
    let create_empty_ctx_tree = OpTree::new(Operation::CreateContext {
        id: ctx_id,
        name: "C1".into(),
        capacity: 10,
    });
    let state_empty_ctx = interpreter
        .execute(&create_empty_ctx_tree, state)
        .value
        .unwrap();
    let result_fail_node = interpreter.execute(&update_tree, state_empty_ctx);
    match result_fail_node.error.unwrap() {
        ModelValidationError::UpdateNodeError { .. } => (),
        _ => panic!("Expected UpdateNodeError"),
    }
}

#[test]
fn test_delete_contextoid_from_context() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();
    let ctx_id = 100;
    let node_id = 1;
    let node = create_dummy_contextoid(node_id);

    // Prepare state
    let root = OpTree::new(Operation::Sequence)
        .add_child(OpTree::new(Operation::CreateContext {
            id: ctx_id,
            name: "C1".into(),
            capacity: 10,
        }))
        .add_child(OpTree::new(Operation::AddContextoidToContext {
            context_id: ctx_id,
            contextoid: node,
        }));

    let state_prepared = interpreter.execute(&root, state.clone()).value.unwrap();

    let delete_op = Operation::DeleteContextoidFromContext {
        context_id: ctx_id,
        contextoid_id: node_id,
    };
    let delete_tree = OpTree::new(delete_op.clone());

    // Success
    let result = interpreter.execute(&delete_tree, state_prepared);
    assert!(result.value.is_some());
    assert!(
        result
            .value
            .unwrap()
            .contexts
            .get(&ctx_id)
            .unwrap()
            .get_node_index_by_id(node_id)
            .is_none()
    );

    // Failure: Context Not Found
    let result_fail_ctx = interpreter.execute(&delete_tree, state.clone());
    match result_fail_ctx.error.unwrap() {
        ModelValidationError::TargetContextNotFound { .. } => (),
        _ => panic!("Expected TargetContextNotFound"),
    }

    // Failure: Node Not Found
    let create_empty_ctx_tree = OpTree::new(Operation::CreateContext {
        id: ctx_id,
        name: "C1".into(),
        capacity: 10,
    });
    let state_empty_ctx = interpreter
        .execute(&create_empty_ctx_tree, state)
        .value
        .unwrap();
    let result_fail_node = interpreter.execute(&delete_tree, state_empty_ctx);
    match result_fail_node.error.unwrap() {
        ModelValidationError::RemoveNodeError { .. } => (),
        _ => panic!("Expected RemoveNodeError"),
    }
}

#[test]
fn test_noop_and_sequence() {
    let interpreter = Interpreter::new();
    let state = create_sys_state();

    // NoOp does nothing
    let no_op_tree = OpTree::new(Operation::NoOp);
    let result_noop = interpreter.execute(&no_op_tree, state.clone());
    assert!(result_noop.value.is_some());
    assert!(result_noop.value.unwrap().causaloids.is_empty());

    // Sequence execution
    let id1 = 1;
    let id2 = 2;
    let seq_tree = OpTree::new(Operation::Sequence)
        .add_child(OpTree::new(Operation::CreateCausaloid(
            id1,
            create_dummy_causaloid(id1),
        )))
        .add_child(OpTree::new(Operation::CreateCausaloid(
            id2,
            create_dummy_causaloid(id2),
        )));

    let result_seq = interpreter.execute(&seq_tree, state.clone());
    assert!(result_seq.value.is_some());
    let final_state = result_seq.value.unwrap();
    assert!(final_state.causaloids.contains_key(&id1));
    assert!(final_state.causaloids.contains_key(&id2));

    // Sequence Short-circuit
    // Create id1, then try to create id1 again (fail)
    let fail_seq_tree = OpTree::new(Operation::Sequence)
        .add_child(OpTree::new(Operation::CreateCausaloid(
            id1,
            create_dummy_causaloid(id1),
        )))
        .add_child(OpTree::new(Operation::CreateCausaloid(
            id1,
            create_dummy_causaloid(id1),
        )));

    let result_fail_seq = interpreter.execute(&fail_seq_tree, state);
    assert!(result_fail_seq.error.is_some());
    match result_fail_seq.error.unwrap() {
        ModelValidationError::DuplicateCausaloidID { .. } => (),
        _ => panic!("Expected DuplicateCausaloidID"),
    }
}
