/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils_generator::{
    MockData, TestCausaloid, TestContext, TestContextoid, TestModel,
};
use deep_causality::{
    CausalEffectLog, CausalFnOutput, CausalityError, ContextoidType, Datable, Identifiable, Root,
};
use std::sync::{Arc, RwLock};

#[test]
fn test_mock_data() {
    let mock = MockData { id: 1, data: 10 };
    assert_eq!(mock.id(), 1);
    assert_eq!(mock.get_data(), 10);

    let mut mock = MockData { id: 1, data: 10 };
    mock.set_data(20);
    assert_eq!(mock.get_data(), 20);
}

#[test]
fn test_mock_data_default() {
    let mock = MockData::default();
    assert_eq!(mock.id(), 0);
    assert_eq!(mock.get_data(), 0);
}

#[test]
fn test_test_causaloid() {
    let id = 1;
    let description = "test";
    fn causal_fn(_: bool) -> Result<CausalFnOutput<bool>, CausalityError> {
        Ok(CausalFnOutput {
            output: true,
            log: CausalEffectLog::new(),
        })
    }

    let causaloid = Arc::new(TestCausaloid::new(id, causal_fn, description));

    assert_eq!(causaloid.id(), id);
    assert_eq!(causaloid.description(), description);
}
#[test]
fn test_test_context() {
    let id = 1;
    let name = "test";
    let context = Arc::new(TestContext::with_capacity(id, name, 12));

    assert_eq!(context.id(), id);
    assert_eq!(context.name(), name);
}

#[test]
fn test_test_contextoid() {
    let id = 1;
    let contextoid = TestContextoid::new(id, ContextoidType::Root(Root::new(1)));
    assert_eq!(contextoid.id(), id);
}

#[test]
fn test_test_model() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    fn causal_fn(_: bool) -> Result<CausalFnOutput<bool>, CausalityError> {
        Ok(CausalFnOutput {
            output: false,
            log: CausalEffectLog::new(),
        })
    }

    let causaloid = Arc::new(TestCausaloid::new(id, causal_fn, "test"));
    let context = Some(Arc::new(RwLock::new(TestContext::with_capacity(
        id, "", 12,
    ))));

    let model = TestModel::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
}
