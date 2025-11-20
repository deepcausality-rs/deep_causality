/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ContextuableGraph, Identifiable};

use deep_causality::{
    BaseSymbol, CausalEffectLog, CausalFnOutput, CausalSystemState, Causaloid, Context, Contextoid,
    ContextoidType, Data, EuclideanSpace, EuclideanSpacetime, EuclideanTime, Interpreter,
    ModelValidationError, OpStatus, OpTree, Operation,
};

// Type aliases for testing
type TestData = Data<f64>;
type TestSpace = EuclideanSpace;
type TestTime = EuclideanTime;
type TestSpacetime = EuclideanSpacetime;
type TestSymbol = BaseSymbol;

// Generic type parameters for CausalSystemState, Causaloid, Context, Contextoid
type I = (); // Input type for Causaloid
type O = (); // Output type for Causaloid
type D = TestData;
type S = TestSpace;
type T = TestTime;
type ST = TestSpacetime;
type Sym = TestSymbol;
type VS = f64; // Value type for Space
type VT = f64; // Value type for Time

type TestCausaloid = Causaloid<I, O, D, S, T, ST, Sym, VS, VT>;
type TestContext = Context<D, S, T, ST, Sym, VS, VT>;
type TestContextoid = Contextoid<D, S, T, ST, Sym, VS, VT>;

// Helper functions for common test scenarios

fn create_dummy_causaloid(id: u64) -> TestCausaloid {
    TestCausaloid::new(
        id,
        // Dummy CausalFn: A simple function that takes I and returns Ok(CausalFnOutput<O>)
        |_: I| Ok(CausalFnOutput::new((), CausalEffectLog::new())),
        format!("Causaloid_{}", id).as_str(),
    )
}

fn create_dummy_context(id: u64, name: &str, capacity: u32) -> TestContext {
    TestContext::with_capacity(id, name, capacity as usize)
}

fn create_dummy_contextoid(id: u64) -> TestContextoid {
    TestContextoid::new(id, ContextoidType::Datoid(TestData::new(id, 1.0)))
}

fn initial_test_state() -> CausalSystemState<I, O, D, S, T, ST, Sym, VS, VT> {
    CausalSystemState::new()
}

// Test Interpreter construction and state properties
#[test]
fn test_causal_system_state_new() {
    let state = initial_test_state();
    assert_eq!(state.causaloids.len(), 0);
    assert_eq!(state.contexts.len(), 0);
}

#[test]
fn test_causal_system_state_clone() {
    let state1 = initial_test_state();
    let state2 = state1.clone();
    assert_eq!(state1.causaloids.len(), state2.causaloids.len());
    assert_eq!(state1.contexts.len(), state2.contexts.len());
}

#[test]
fn test_interpreter_new() {
    let _interpreter = Interpreter::new();
    // Interpreter is a zero-sized type, just verify it can be created
}

#[test]
fn test_interpreter_is_stateless() {
    let interpreter1 = Interpreter::new();
    let interpreter2 = Interpreter::new();
    // Both interpreters should be identical (stateless)
    let _ = (interpreter1, interpreter2);
}

#[test]
fn test_causal_system_state_debug() {
    let state = initial_test_state();
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("CausalSystemState"));
}

// --- Interpreter Operations Tests ---

// Test Operation::NoOp
#[test]
fn test_walk_noop() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();
    let op_node = OpTree::new(Operation::NoOp);

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert!(result.logs.is_empty());
    assert_eq!(result.value.as_ref().unwrap().causaloids.len(), 0);
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 0);
}

// Test Operation::CreateCausaloid
#[test]
fn test_walk_create_causaloid_success() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();
    let causaloid = create_dummy_causaloid(1);
    let op_node = OpTree::new(Operation::CreateCausaloid(1, causaloid.clone()));

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "CreateCausaloid".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    assert_eq!(result.value.as_ref().unwrap().causaloids.len(), 1);
    assert!(result.value.as_ref().unwrap().causaloids.contains_key(&1));
}

#[test]
fn test_walk_create_causaloid_duplicate_id() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    let causaloid = create_dummy_causaloid(1);
    initial_state.causaloids.insert(1, causaloid.clone()); // Pre-add causaloid

    let op_node = OpTree::new(Operation::CreateCausaloid(1, causaloid));

    let result = interpreter.execute(&op_node, initial_state.clone()); // Pass cloned state

    assert!(result.value.is_some()); // Value should still be original state
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::DuplicateCausaloidID { id: 1 }
    ));
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "CreateCausaloid".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(result.value.as_ref().unwrap().causaloids.len(), 1);
    assert!(result.value.as_ref().unwrap().causaloids.contains_key(&1));
}

// Test Operation::UpdateCausaloid
#[test]
fn test_walk_update_causaloid_success() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    let causaloid1 = create_dummy_causaloid(1);
    initial_state.causaloids.insert(1, causaloid1);

    let updated_causaloid = create_dummy_causaloid(1); // Assuming some change
    let op_node = OpTree::new(Operation::UpdateCausaloid(1, updated_causaloid.clone()));

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "UpdateCausaloid".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    assert_eq!(result.value.as_ref().unwrap().causaloids.len(), 1);
    assert!(result.value.as_ref().unwrap().causaloids.contains_key(&1));
}

#[test]
fn test_walk_update_causaloid_not_found() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();
    let causaloid = create_dummy_causaloid(1);
    let op_node = OpTree::new(Operation::UpdateCausaloid(1, causaloid));

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Returns original state
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::UpdateNodeError { .. }
    ));
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "UpdateCausaloid".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(result.value.as_ref().unwrap().causaloids.len(), 0);
}

// Test Operation::DeleteCausaloid
#[test]
fn test_walk_delete_causaloid_success() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    initial_state
        .causaloids
        .insert(1, create_dummy_causaloid(1));

    let op_node = OpTree::new(Operation::DeleteCausaloid(1));

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "DeleteCausaloid".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    assert_eq!(result.value.as_ref().unwrap().causaloids.len(), 0);
}

#[test]
fn test_walk_delete_causaloid_not_found() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();
    let op_node = OpTree::new(Operation::DeleteCausaloid(1));

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Returns original state
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::RemoveNodeError { .. }
    ));
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "DeleteCausaloid".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(result.value.as_ref().unwrap().causaloids.len(), 0);
}

// Test Operation::CreateContext
#[test]
fn test_walk_create_context_success() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();
    let op_node = OpTree::new(Operation::CreateContext {
        id: 1,
        name: "TestContext".to_string(),
        capacity: 10,
    });

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "CreateContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 1);
    assert!(result.value.as_ref().unwrap().contexts.contains_key(&1));
}

#[test]
fn test_walk_create_context_duplicate_id() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    initial_state
        .contexts
        .insert(1, create_dummy_context(1, "Existing", 10));

    let op_node = OpTree::new(Operation::CreateContext {
        id: 1,
        name: "TestContext".to_string(),
        capacity: 10,
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Returns original state
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::DuplicateContextId { id: 1 }
    ));
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "CreateContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 1);
}

// Test Operation::CreateExtraContext
#[test]
fn test_walk_create_extra_context_success() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    // Add parent context
    initial_state
        .contexts
        .insert(0, create_dummy_context(0, "Parent", 10));

    let op_node = OpTree::new(Operation::CreateExtraContext {
        context_id: 0,
        extra_context_id: 1,
        capacity: 10,
    });

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "CreateExtraContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 2);
    assert!(result.value.as_ref().unwrap().contexts.contains_key(&1));
}

#[test]
fn test_walk_create_extra_context_duplicate_id() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    // Add parent context
    initial_state
        .contexts
        .insert(0, create_dummy_context(0, "Parent", 10));
    initial_state
        .contexts
        .insert(1, create_dummy_context(1, "ExistingExtra", 10));

    let op_node = OpTree::new(Operation::CreateExtraContext {
        context_id: 0,
        extra_context_id: 1,
        capacity: 10,
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Returns original state
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::DuplicateContextId { id: 1 }
    ));
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "CreateExtraContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 2);
}
// Test Operation::UpdateContext
#[test]
fn test_walk_update_context_success_with_name() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    initial_state
        .contexts
        .insert(1, create_dummy_context(1, "OldName", 10));

    let op_node = OpTree::new(Operation::UpdateContext {
        id: 1,
        new_name: Some("NewName".to_string()),
    });

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "UpdateContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 1);
    assert_eq!(
        result.value.as_ref().unwrap().contexts[&1].name(),
        "NewName"
    );
}

#[test]
fn test_walk_update_context_success_no_name() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    initial_state
        .contexts
        .insert(1, create_dummy_context(1, "OldName", 10));

    let op_node = OpTree::new(Operation::UpdateContext {
        id: 1,
        new_name: None,
    });

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "UpdateContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 1);
    assert_eq!(
        result.value.as_ref().unwrap().contexts[&1].name(),
        "OldName"
    ); // Name should not change
}

#[test]
fn test_walk_update_context_not_found() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();

    let op_node = OpTree::new(Operation::UpdateContext {
        id: 1,
        new_name: Some("NewName".to_string()),
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Returns original state
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::TargetContextNotFound { id: 1 }
    ));
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "UpdateContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 0);
}

// Test Operation::DeleteContext
#[test]
fn test_walk_delete_context_success() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    initial_state
        .contexts
        .insert(1, create_dummy_context(1, "Test", 10));

    let op_node = OpTree::new(Operation::DeleteContext(1));

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "DeleteContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 0);
}

#[test]
fn test_walk_delete_context_not_found() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();
    let op_node = OpTree::new(Operation::DeleteContext(1));

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Returns original state
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::TargetContextNotFound { id: 1 }
    ));
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "DeleteContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 0);
}

// Test Operation::AddContextoidToContext
#[test]
fn test_walk_add_contextoid_success() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    initial_state
        .contexts
        .insert(1, create_dummy_context(1, "Test", 10));
    let contextoid = create_dummy_contextoid(101);

    let op_node = OpTree::new(Operation::AddContextoidToContext {
        context_id: 1,
        contextoid: contextoid.clone(),
    });

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "AddContextoidToContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    let final_state = result.value.as_ref().unwrap();
    assert!(final_state.contexts[&1].get_node_index_by_id(101).is_some());
}

#[test]
fn test_walk_add_contextoid_context_not_found() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();
    let contextoid = create_dummy_contextoid(101);

    let op_node = OpTree::new(Operation::AddContextoidToContext {
        context_id: 1,
        contextoid: contextoid.clone(),
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Value is Some on error in this case
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::TargetContextNotFound { id: 1 }
    ));
    assert_eq!(result.logs.len(), 0); // No logs added in interpreter when context not found
    // Ensure state is unchanged
    assert_eq!(initial_state.contexts.len(), 0);
}

#[test]
fn test_walk_add_contextoid_duplicate() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    let mut context = create_dummy_context(1, "Test", 10);
    let contextoid = create_dummy_contextoid(101);
    context.add_node(contextoid.clone()).unwrap(); // Add it once
    initial_state.contexts.insert(1, context);

    let op_node = OpTree::new(Operation::AddContextoidToContext {
        context_id: 1,
        contextoid: contextoid.clone(),
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Value is Some on error
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::AddContextoidError { .. }
    ));
    assert_eq!(result.logs.len(), 1); // Log only created if context found
    // Ensure state is unchanged regarding causaloids
    assert_eq!(initial_state.contexts[&1].number_of_nodes(), 1);
}

// Test Operation::UpdateContextoidInContext
#[test]
fn test_walk_update_contextoid_success() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    let mut context = create_dummy_context(1, "Test", 10);
    let existing_contextoid = create_dummy_contextoid(101);
    context.add_node(existing_contextoid.clone()).unwrap();
    initial_state.contexts.insert(1, context);

    let updated_contextoid = create_dummy_contextoid(101); // Assuming some internal change
    let op_node = OpTree::new(Operation::UpdateContextoidInContext {
        context_id: 1,
        existing_contextoid: updated_contextoid.id(),
        new_contextoid: updated_contextoid,
    });

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "UpdateContextoidInContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    let final_state = result.value.as_ref().unwrap();
    assert!(final_state.contexts[&1].get_node_index_by_id(101).is_some());
}

#[test]
fn test_walk_update_contextoid_context_not_found() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();
    let contextoid = create_dummy_contextoid(101);

    let op_node = OpTree::new(Operation::UpdateContextoidInContext {
        context_id: 1,
        existing_contextoid: 101,
        new_contextoid: contextoid,
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Value is Some on error
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::TargetContextNotFound { id: 1 }
    ));
    assert_eq!(result.logs.len(), 1); // Log added in interpreter
    assert_eq!(
        result.logs.entries[0].operation_name,
        "UpdateContextoidInContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(initial_state.contexts.len(), 0);
}

#[test]
fn test_walk_update_contextoid_not_found_in_context() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    initial_state
        .contexts
        .insert(1, create_dummy_context(1, "Test", 10));

    let contextoid = create_dummy_contextoid(101);
    let op_node = OpTree::new(Operation::UpdateContextoidInContext {
        context_id: 1,
        existing_contextoid: 999, // Non-existent contextoid
        new_contextoid: contextoid,
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Value is Some on error
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::UpdateNodeError { .. }
    ));
    assert_eq!(result.logs.len(), 1); // Log added in interpreter
    assert_eq!(
        result.logs.entries[0].operation_name,
        "UpdateContextoidInContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(initial_state.contexts[&1].number_of_nodes(), 0); // Context is empty
}

// Test Operation::DeleteContextoidFromContext
#[test]
fn test_walk_delete_contextoid_success() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    let mut context = create_dummy_context(1, "Test", 10);
    context.add_node(create_dummy_contextoid(101)).unwrap();
    initial_state.contexts.insert(1, context);

    let op_node = OpTree::new(Operation::DeleteContextoidFromContext {
        context_id: 1,
        contextoid_id: 101,
    });

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "DeleteContextoidFromContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Success);
    let final_state = result.value.as_ref().unwrap();
    assert!(!final_state.contexts[&1].contains_node(101usize));
}

#[test]
fn test_walk_delete_contextoid_context_not_found() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();

    let op_node = OpTree::new(Operation::DeleteContextoidFromContext {
        context_id: 1,
        contextoid_id: 101,
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Value is Some on error
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::TargetContextNotFound { id: 1 }
    ));
    assert_eq!(result.logs.len(), 1); // Log added in interpreter
    assert_eq!(
        result.logs.entries[0].operation_name,
        "DeleteContextoidFromContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(initial_state.contexts.len(), 0);
}

#[test]
fn test_walk_delete_contextoid_not_found_in_context() {
    let interpreter = Interpreter::new();
    let mut initial_state = initial_test_state();
    initial_state
        .contexts
        .insert(1, create_dummy_context(1, "Test", 10));

    let op_node = OpTree::new(Operation::DeleteContextoidFromContext {
        context_id: 1,
        contextoid_id: 999, // Non-existent contextoid
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Value is Some on error
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::RemoveNodeError { .. }
    ));
    assert_eq!(result.logs.len(), 1); // Log added in interpreter
    assert_eq!(
        result.logs.entries[0].operation_name,
        "DeleteContextoidFromContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(initial_state.contexts[&1].number_of_nodes(), 0); // Context is still empty
}

// Test Operation::Sequence
#[test]
fn test_walk_sequence_empty() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();
    let op_node = OpTree::with_children(Operation::Sequence, vec![]);

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert!(result.logs.is_empty());
    assert_eq!(result.value.as_ref().unwrap().causaloids.len(), 0);
    assert_eq!(result.value.as_ref().unwrap().contexts.len(), 0);
}

#[test]
fn test_walk_sequence_success() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();

    let op1 = OpTree::new(Operation::CreateCausaloid(1, create_dummy_causaloid(1)));
    let op2 = OpTree::new(Operation::CreateContext {
        id: 10,
        name: "SeqContext".to_string(),
        capacity: 5,
    });
    let op_node = OpTree::with_children(Operation::Sequence, vec![op1, op2]);

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.logs.len(), 2);
    assert!(
        result
            .logs
            .iter()
            .any(|e| e.operation_name == "CreateCausaloid")
    );
    assert!(
        result
            .logs
            .iter()
            .any(|e| e.operation_name == "CreateContext")
    );
    let final_state = result.value.as_ref().unwrap();
    assert_eq!(final_state.causaloids.len(), 1);
    assert_eq!(final_state.contexts.len(), 1);
}

#[test]
fn test_walk_sequence_error_short_circuit() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();

    let op1 = OpTree::new(Operation::CreateContext {
        id: 1,
        name: "FirstContext".to_string(),
        capacity: 5,
    }); // Success
    let op2 = OpTree::new(Operation::CreateContext {
        id: 1, // Duplicate ID, will cause error
        name: "SeqContext".to_string(),
        capacity: 5,
    });
    let op3 = OpTree::new(Operation::CreateCausaloid(2, create_dummy_causaloid(2))); // This should not execute

    let op_node = OpTree::with_children(Operation::Sequence, vec![op1, op2, op3]);

    let result = interpreter.execute(&op_node, initial_state);

    assert!(result.value.is_none()); // Returns None before error
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::DuplicateContextId { id: 1 }
    ));
    assert_eq!(result.logs.len(), 2); // Log for op1 and op2 (failure)
    assert!(
        result
            .logs
            .iter()
            .any(|e| e.operation_name == "CreateContext" && e.status == OpStatus::Success)
    );
    assert!(
        result
            .logs
            .iter()
            .any(|e| e.operation_name == "CreateContext" && e.status == OpStatus::Failure)
    );
}

#[test]
fn test_walk_sequence_log_aggregation() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();

    let op1 = OpTree::new(Operation::CreateCausaloid(1, create_dummy_causaloid(1)));
    let op2 = OpTree::new(Operation::CreateContext {
        id: 10,
        name: "SeqContext".to_string(),
        capacity: 5,
    });
    let op_node = OpTree::with_children(Operation::Sequence, vec![op1, op2]);

    let result = interpreter.execute(&op_node, initial_state);

    assert_eq!(result.logs.len(), 2);
    assert!(
        result
            .logs
            .iter()
            .any(|e| e.operation_name == "CreateCausaloid" && e.target_id == "1")
    );
    assert!(
        result
            .logs
            .iter()
            .any(|e| e.operation_name == "CreateContext" && e.target_id == "10")
    );
}

#[test]
fn test_walk_create_extra_context_parent_not_found() {
    let interpreter = Interpreter::new();
    let initial_state = initial_test_state();

    let op_node = OpTree::new(Operation::CreateExtraContext {
        context_id: 999, // Non-existent parent context
        extra_context_id: 1,
        capacity: 10,
    });

    let result = interpreter.execute(&op_node, initial_state.clone());

    assert!(result.value.is_some()); // Returns original state
    assert!(result.error.is_some());
    assert!(matches!(
        result.error.unwrap(),
        ModelValidationError::TargetContextNotFound { id: 999 }
    ));
    assert_eq!(result.logs.len(), 1);
    assert_eq!(
        result.logs.entries[0].operation_name,
        "CreateExtraContext".to_string()
    );
    assert_eq!(result.logs.entries[0].status, OpStatus::Failure);
    // Ensure state is unchanged
    assert_eq!(initial_state.contexts.len(), 0);
}
