/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseContext, BaseContextoid, Causaloid, Context, Model, ModelValidationError, OpStatus,
    Operation, PropagatingEffect,
};

use deep_causality_ast::ConstTree;
use std::sync::{Arc, RwLock};

// Use concrete types from deep_causality
type TestInput = f64;
type TestOutput = f64;
type TestContext = BaseContext;
type TestNode = BaseContextoid;

// Model<I, O, C> - C must be the bare Context struct, not Arc<RwLock<...>>
type TestModel = Model<TestInput, TestOutput, TestContext>;

// Operation<I, O, C, N> - C must be the bare Context struct here too, based on Model::evolve signature
type TestOperation = Operation<TestInput, TestOutput, TestContext, TestNode>;

// Helper function for Causaloid. Must be a fn pointer.
fn base_causal_fn(_input: TestInput) -> PropagatingEffect<TestOutput> {
    PropagatingEffect::from_value(0.0)
}

fn new_causal_fn(_input: TestInput) -> PropagatingEffect<TestOutput> {
    PropagatingEffect::from_value(1.0)
}

#[test]
fn test_hkt_generative_system_evolve() {
    // 1. Create a base model
    // Causaloid expects Context generic to be Arc<RwLock<TestContext>> because Model wraps C in Arc<RwLock>
    let causaloid: Arc<Causaloid<TestInput, TestOutput, (), Arc<RwLock<TestContext>>>> =
        Arc::new(Causaloid::new(1, base_causal_fn, "Base Causaloid"));

    let context = Context::with_capacity(100, "BaseContext", 10);
    let context_arc = Arc::new(RwLock::new(context));

    let model = TestModel::new(
        1,
        "Author",
        "Description",
        None,
        causaloid,
        Some(context_arc),
    );

    // 2. Define an OpTree
    let op1 = TestOperation::UpdateContext {
        id: 100,
        new_name: Some("UpdatedContext".to_string()),
    };

    let op2 = TestOperation::CreateExtraContext {
        context_id: 100,
        extra_context_id: 200,
        capacity: 5,
    };

    let new_causaloid: Causaloid<TestInput, TestOutput, (), Arc<RwLock<TestContext>>> =
        Causaloid::new(1, new_causal_fn, "New Causaloid");

    let op3 = TestOperation::UpdateCausaloid(1, new_causaloid);

    // Construct Tree
    let leaf_op1 = ConstTree::new(op1);
    let leaf_op2 = ConstTree::new(op2);
    let leaf_op3 = ConstTree::new(op3);

    let root_op = TestOperation::Sequence;
    let op_tree = ConstTree::with_children(root_op, vec![leaf_op1, leaf_op2, leaf_op3]);

    // 3. Evolve
    let result = model.evolve(&op_tree);

    // 4. Assertions
    assert!(result.is_ok(), "Evolve should succeed: {:?}", result.err());

    let (new_model, logs) = result.unwrap();

    // Verify Context Name Update
    let ctx_opt = new_model.context();
    let ctx_lock = ctx_opt.as_ref().unwrap();
    let ctx = ctx_lock.read().unwrap();
    assert_eq!(ctx.name(), "UpdatedContext");

    // Verify Logs
    let log_entries = logs.entries;
    assert!(
        log_entries
            .iter()
            .any(|e| e.operation_name == "UpdateContext" && e.status == OpStatus::Success)
    );
    assert!(
        log_entries
            .iter()
            .any(|e| e.operation_name == "CreateExtraContext" && e.status == OpStatus::Success)
    );
    assert!(
        log_entries
            .iter()
            .any(|e| e.operation_name == "UpdateCausaloid" && e.status == OpStatus::Success)
    );
}

#[test]
fn test_evolve_error_causaloid_lost() {
    // 1. Create a base model
    let causaloid: Arc<Causaloid<TestInput, TestOutput, (), Arc<RwLock<TestContext>>>> =
        Arc::new(Causaloid::new(1, base_causal_fn, "Base Causaloid"));
    let model = TestModel::new(1, "Author", "Description", None, causaloid, None);

    // 2. Define an OpTree that deletes the main causaloid
    let op = TestOperation::DeleteCausaloid(1);
    let op_tree = ConstTree::new(op);

    // 3. Evolve
    let result = model.evolve(&op_tree);

    // 4. Assertions
    assert!(result.is_err(), "Evolve should fail");
}

#[test]
fn test_evolve_error_from_interpreter() {
    // 1. Create a base model with a context
    let causaloid: Arc<Causaloid<TestInput, TestOutput, (), Arc<RwLock<TestContext>>>> =
        Arc::new(Causaloid::new(1, base_causal_fn, "Base Causaloid"));
    let context = Context::with_capacity(100, "BaseContext", 10);
    let context_arc = Arc::new(RwLock::new(context));
    let model = TestModel::new(
        1,
        "Author",
        "Description",
        None,
        causaloid,
        Some(context_arc),
    );

    // 2. Define an OpTree that will cause an error in the interpreter
    // (e.g., creating a duplicate context)
    let op = TestOperation::CreateContext {
        id: 100, // Duplicate ID
        name: "Duplicate Context".to_string(),
        capacity: 5,
    };
    let op_tree = ConstTree::new(op);

    // 3. Evolve
    let result = model.evolve(&op_tree);

    // 4. Assertions
    assert!(result.is_err(), "Evolve should fail");
    let err = result.err().unwrap();
    assert!(matches!(
        err,
        ModelValidationError::DuplicateContextId { .. }
    ));
}
