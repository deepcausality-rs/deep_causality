/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;
use deep_causality::utils_test::*;
use std::hash::Hash;

// Mock user-defined Generatable enum for testing the Evolve variant
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MockGeneratable {
    CustomAction,
}

// Implement the Generatable trait for the mock enum
impl<D, S, T, ST, SYM, VS, VT, G> Generatable<D, S, T, ST, SYM, VS, VT, G> for MockGeneratable
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    G: Generatable<D, S, T, ST, SYM, VS, VT, G> + Sized,
{
    // This mock implementation is minimal, as its internal logic is not the focus of these tests.
}

// Define a type alias for the specific GenerativeOutput used in tests for brevity.
type TestGenerativeOutput = GenerativeOutput<
    MockData,
    MockSpatial,
    MockTemporal,
    MockSpaceTemporal,
    MockSymbolic,
    MockVS,
    MockVT,
    MockGeneratable,
>;

#[test]
fn test_noop() {
    let output = TestGenerativeOutput::NoOp;
    assert!(matches!(output, TestGenerativeOutput::NoOp));

    // Verify Clone trait
    let cloned_output = output.clone();
    // Verify Debug trait by comparing debug strings
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_create_causaloid() {
    let causaloid = get_test_causaloid();
    let output = TestGenerativeOutput::CreateCausaloid(1, causaloid.clone());

    if let TestGenerativeOutput::CreateCausaloid(id, c) = output.clone() {
        assert_eq!(id, 1);
        assert_eq!(c.id(), causaloid.id());
    } else {
        panic!("Expected CreateCausaloid variant, but got {:?}", output);
    }

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_update_causaloid() {
    let causaloid = get_test_causaloid();
    let output = TestGenerativeOutput::UpdateCausaloid(1, causaloid.clone());

    if let TestGenerativeOutput::UpdateCausaloid(id, c) = output.clone() {
        assert_eq!(id, 1);
        assert_eq!(c.id(), causaloid.id());
    } else {
        panic!("Expected UpdateCausaloid variant, but got {:?}", output);
    }

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_delete_causaloid() {
    let output = TestGenerativeOutput::DeleteCausaloid(1);
    assert!(matches!(output, TestGenerativeOutput::DeleteCausaloid(1)));

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_create_base_context() {
    let output = TestGenerativeOutput::CreateBaseContext {
        id: 1,
        name: "test_context".to_string(),
        capacity: 100,
    };

    if let TestGenerativeOutput::CreateBaseContext { id, ref name, capacity } = output.clone() {
        assert_eq!(id, 1);
        assert_eq!(name, "test_context");
        assert_eq!(capacity, 100);
    } else {
        panic!("Expected CreateBaseContext variant, but got {:?}", output);
    }

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_create_extra_context() {
    let output = TestGenerativeOutput::CreateExtraContext {
        extra_context_id: 2,
        capacity: 50,
    };

    if let TestGenerativeOutput::CreateExtraContext { extra_context_id, capacity } = output.clone()
    {
        assert_eq!(extra_context_id, 2);
        assert_eq!(capacity, 50);
    } else {
        panic!("Expected CreateExtraContext variant, but got {:?}", output);
    }

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_update_context() {
    let output = TestGenerativeOutput::UpdateContext {
        id: 1,
        new_name: Some("new_name".to_string()),
    };

    if let TestGenerativeOutput::UpdateContext { id, ref new_name } = output.clone() {
        assert_eq!(id, 1);
        assert_eq!(new_name, &Some("new_name".to_string()));
    } else {
        panic!("Expected UpdateContext variant, but got {:?}", output);
    }

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_delete_context() {
    let output = TestGenerativeOutput::DeleteContext { id: 1 };
    assert!(matches!(output, TestGenerativeOutput::DeleteContext { id: 1 }));

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_add_contextoid_to_context() {
    let contextoid = get_test_contextoid();
    let output = TestGenerativeOutput::AddContextoidToContext {
        context_id: 1,
        contextoid: contextoid.clone(),
    };

    if let TestGenerativeOutput::AddContextoidToContext { context_id, contextoid: c } =
        output.clone()
    {
        assert_eq!(context_id, 1);
        assert_eq!(c.id(), contextoid.id());
    } else {
        panic!("Expected AddContextoidToContext variant, but got {:?}", output);
    }

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_update_contextoid_in_context() {
    let contextoid = get_test_contextoid();
    let output = TestGenerativeOutput::UpdateContextoidInContext {
        context_id: 1,
        existing_contextoid: 2,
        new_contextoid: contextoid.clone(),
    };

    if let TestGenerativeOutput::UpdateContextoidInContext {
        context_id,
        existing_contextoid,
        new_contextoid: c,
    } = output.clone()
    {
        assert_eq!(context_id, 1);
        assert_eq!(existing_contextoid, 2);
        assert_eq!(c.id(), contextoid.id());
    } else {
        panic!("Expected UpdateContextoidInContext variant, but got {:?}", output);
    }

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_delete_contextoid_from_context() {
    let output = TestGenerativeOutput::DeleteContextoidFromContext {
        context_id: 1,
        contextoid_id: 2,
    };

    if let TestGenerativeOutput::DeleteContextoidFromContext { context_id, contextoid_id } =
        output.clone()
    {
        assert_eq!(context_id, 1);
        assert_eq!(contextoid_id, 2);
    } else {
        panic!("Expected DeleteContextoidFromContext variant, but got {:?}", output);
    }

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}

#[test]
fn test_composite() {
    let noop = TestGenerativeOutput::NoOp;
    let delete = TestGenerativeOutput::DeleteCausaloid(5);
    let outputs = vec![noop.clone(), delete.clone()];
    let composite_output = TestGenerativeOutput::Composite(outputs);

    if let TestGenerativeOutput::Composite(inner_outputs) = composite_output.clone() {
        assert_eq!(inner_outputs.len(), 2);
        assert!(matches!(inner_outputs[0], TestGenerativeOutput::NoOp));
        assert!(matches!(inner_outputs[1], TestGenerativeOutput::DeleteCausaloid(5)));
    } else {
        panic!("Expected Composite variant, but got {:?}", composite_output);
    }

    let cloned_output = composite_output.clone();
    assert_eq!(format!("{:?}", composite_output), format!("{:?}", cloned_output));
}

#[test]
fn test_evolve() {
    let custom_action = MockGeneratable::CustomAction;
    let output = TestGenerativeOutput::Evolve(custom_action.clone());

    if let TestGenerativeOutput::Evolve(action) = output.clone() {
        assert_eq!(action, custom_action);
    } else {
        panic!("Expected Evolve variant, but got {:?}", output);
    }

    let cloned_output = output.clone();
    assert_eq!(format!("{:?}", output), format!("{:?}", cloned_output));
}
