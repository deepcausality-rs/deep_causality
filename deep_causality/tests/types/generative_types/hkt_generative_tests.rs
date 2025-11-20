/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseSymbol, CausalEffectLog, CausalFnOutput, Causaloid, Context, Data, EffectValue,
    EuclideanSpace, EuclideanSpacetime, EuclideanTime, Model, OpStatus, Operation,
};

use deep_causality_ast::ConstTree;
use std::sync::Arc;

// Use concrete types from deep_causality instead of mocks
type TestModel = Model<
    EffectValue,
    EffectValue,
    Data<f64>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    f64,
    f64,
>;

type TestOperation = Operation<
    EffectValue,
    EffectValue,
    Data<f64>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    f64,
    f64,
>;

#[test]
fn test_hkt_generative_system_evolve() {
    // 1. Create a base model
    // CausalFn likely takes 1 arg (input) and returns Result<Output, Error>
    let causaloid = Arc::new(Causaloid::new(
        1,
        |_input: EffectValue| {
            Ok(CausalFnOutput::new(
                EffectValue::from(0.0),
                CausalEffectLog::default(),
            ))
        },
        "Base Causaloid",
    ));
    let context = Context::with_capacity(100, "BaseContext", 10);
    let context_arc = Arc::new(std::sync::RwLock::new(context));

    let model = TestModel::new(
        1,
        "Author",
        "Description",
        None,
        causaloid,
        Some(context_arc),
    );

    // 2. Define an OpTree
    // Sequence:
    // 1. Update Context Name
    // 2. Create Extra Context
    // 3. Create Causaloid (update existing one)

    let op1 = TestOperation::UpdateContext {
        id: 100, // Assuming base context ID is 100
        new_name: Some("UpdatedContext".to_string()),
    };

    let op2 = TestOperation::CreateExtraContext {
        context_id: 100,
        extra_context_id: 200,
        capacity: 5,
    };

    let new_causaloid = Causaloid::new(
        1,
        |_input: EffectValue| {
            Ok(CausalFnOutput::new(
                EffectValue::from(1.0),
                CausalEffectLog::default(),
            ))
        },
        "New Causaloid",
    );
    let op3 = TestOperation::UpdateCausaloid(1, new_causaloid);

    // Construct Tree manually

    let leaf_op3 = ConstTree::new(op3);
    let leaf_op2 = ConstTree::new(op2);
    let leaf_op1 = ConstTree::new(op1);

    let root_op = TestOperation::Sequence;
    let op_tree = ConstTree::with_children(root_op, vec![leaf_op1, leaf_op2, leaf_op3]);

    // 3. Evolve
    let result = model.evolve(&op_tree);

    // 4. Assertions
    assert!(result.is_ok(), "Evolve should succeed: {:?}", result.err());

    let (new_model, logs) = result.unwrap();

    // Verify Context Name Update
    // We need to access the context of the new model.
    let ctx_lock = new_model.context().as_ref().unwrap();
    let ctx = ctx_lock.read().unwrap();
    assert_eq!(ctx.name(), "UpdatedContext");

    // Verify Causaloid Update
    // The new causaloid should have the new function (we can't easily check function equality, but we can check ID or behavior if we could run it)
    // The logs should confirm the update.

    // Verify Logs
    println!("Logs: {:?}", logs);
    // Check for specific log entries
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
