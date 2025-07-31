/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils;
use deep_causality::{AssumptionError, Identifiable, Model, PropagatingEffect, Transferable};
use std::sync::Arc;

#[test]
fn test_assumptions_no_assumptions_err() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = Arc::new(test_utils::get_test_causaloid_deterministic());
    let context = Some(Arc::new(test_utils::get_test_context()));

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_none());

    let data: Vec<PropagatingEffect> = test_utils::get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::Numerical(x))
        .collect();

    // AssumptionError::NoAssumptionsDefined
    let res = model.verify_assumptions(&data);
    assert!(res.is_err());

    let err = res.unwrap_err();
    assert_eq!(err, AssumptionError::NoAssumptionsDefined)
}

#[test]
fn test_assumptions_no_data_err() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = Some(Arc::new(vec![test_utils::get_test_assumption()]));
    let causaloid = Arc::new(test_utils::get_test_causaloid_deterministic());
    let context = Some(Arc::new(test_utils::get_test_context()));

    let model = Model::new(
        id,
        author,
        description,
        assumptions,
        causaloid.clone(),
        context,
    );

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_some());
    assert_eq!(*model.causaloid(), causaloid);

    // AssumptionError::NoDataToTestDefined
    let res = model.verify_assumptions(&[]);
    assert!(res.is_err());

    let err = res.unwrap_err();
    assert_eq!(err, AssumptionError::NoDataToTestDefined);
}

#[test]
fn test_assumptions_assumption_err() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = Some(Arc::new(vec![test_utils::get_test_assumption_error()]));
    let causaloid = Arc::new(test_utils::get_test_causaloid_deterministic());
    let context = None;

    let model = Model::new(
        id,
        author,
        description,
        assumptions,
        causaloid.clone(),
        context,
    );

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_some());
    assert_eq!(*model.causaloid(), causaloid);

    let data: Vec<PropagatingEffect> = test_utils::get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::Numerical(x))
        .collect();

    // Err(AssumptionFailed("Test error"),
    let res = model.verify_assumptions(&data);
    assert!(res.is_err());
}

#[test]
fn test_verify_assumptions_failed() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = Some(Arc::new(vec![test_utils::get_test_assumption_false()]));
    let causaloid = Arc::new(test_utils::get_test_causaloid_deterministic());
    let context = Some(Arc::new(test_utils::get_test_context()));

    let model = Model::new(
        id,
        author,
        description,
        assumptions,
        causaloid.clone(),
        context,
    );

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_some());
    assert_eq!(*model.causaloid(), causaloid);

    let data: Vec<PropagatingEffect> = test_utils::get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::Numerical(x))
        .collect();

    //  AssumptionError::AssumptionFailed("Assumption: id: 2, description: Test assumption that is always false, assumption_tested: true, assumption_valid: false")
    let res = model.verify_assumptions(&data);
    assert!(res.is_err());
}
#[test]
fn test_verify_assumptions_success() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = Some(Arc::new(vec![test_utils::get_test_assumption()]));
    let causaloid = Arc::new(test_utils::get_test_causaloid_deterministic());
    let context = Some(Arc::new(test_utils::get_test_context()));

    let model = Model::new(
        id,
        author,
        description,
        assumptions,
        causaloid.clone(),
        context,
    );

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_some());
    assert_eq!(*model.causaloid(), causaloid);

    let data: Vec<PropagatingEffect> = test_utils::get_test_num_array()
        .iter()
        .map(|&x| PropagatingEffect::Numerical(x))
        .collect();

    let res = model.verify_assumptions(&data);
    assert!(res.is_ok());
}
