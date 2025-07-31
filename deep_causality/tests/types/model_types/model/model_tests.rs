/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::{Identifiable, Model};
use std::sync::Arc;

#[test]
fn test_new() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = Arc::new(test_utils::get_test_causaloid());
    let context = Some(Arc::new(test_utils::get_test_context()));

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = Arc::new(test_utils::get_test_causaloid());
    let context = Some(Arc::new(test_utils::get_test_context()));

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
}

#[test]
fn test_author() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = Arc::new(test_utils::get_test_causaloid());
    let context = Some(Arc::new(test_utils::get_test_context()));

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
}

#[test]
fn test_description() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = Arc::new(test_utils::get_test_causaloid());
    let context = Some(Arc::new(test_utils::get_test_context()));

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
}

#[test]
fn test_causaloid() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = Arc::new(test_utils::get_test_causaloid());
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
    assert!(model.assumptions().is_none());
    assert_eq!(*model.causaloid(), causaloid);
}

#[test]
fn test_context() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = Arc::new(test_utils::get_test_causaloid());
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
    assert!(model.assumptions().is_none());
    assert_eq!(*model.causaloid(), causaloid);
    assert!(model.context().is_some());
    assert_eq!(model.context().clone().unwrap().id(), id);
}

#[test]
fn test_assumptions() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = Arc::new(test_utils::get_test_causaloid());
    let context = Some(Arc::new(test_utils::get_test_context()));

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_none());
}
