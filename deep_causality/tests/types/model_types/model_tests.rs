// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Identifiable, Model};
use deep_causality::types::model_types::Model;

use crate::utils::test_utils::*;

#[test]
fn test_new() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = &get_test_causaloid();
    let binding = get_test_context();
    let context = Some(&binding);

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = &get_test_causaloid();
    let binding = get_test_context();
    let context = Some(&binding);

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(*model.model_id(), id);
}

#[test]
fn test_author() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = &get_test_causaloid();
    let binding = get_test_context();
    let context = Some(&binding);

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(*model.author(), author);
}

#[test]
fn test_description() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = &get_test_causaloid();
    let binding = get_test_context();
    let context = Some(&binding);

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(*model.author(), author);
    assert_eq!(*model.description(), description);
}

#[test]
fn test_assumptions() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = &get_test_causaloid();
    let binding = get_test_context();
    let context = Some(&binding);

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(*model.author(), author);
    assert_eq!(*model.description(), description);
    assert!(model.assumptions().is_none());
}

#[test]
fn test_causaloid() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = &get_test_causaloid();
    let binding = get_test_context();
    let context = Some(&binding);

    let model = Model::new(id, author, description, assumptions, causaloid, context);

    assert_eq!(model.id(), id);
    assert_eq!(*model.author(), author);
    assert_eq!(*model.description(), description);
    assert!(model.assumptions().is_none());
    assert_eq!(*model.causaloid(), causaloid);
}

#[test]
fn test_context() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = &get_test_causaloid();
    let context = get_test_context();

    let model = Model::new(
        id,
        author,
        description,
        assumptions,
        causaloid,
        Some(&context),
    );

    assert_eq!(model.id(), id);
    assert_eq!(*model.author(), author);
    assert_eq!(*model.description(), description);
    assert!(model.assumptions().is_none());
    assert_eq!(*model.causaloid(), causaloid);
    assert!(model.context().is_some());
    assert_eq!(model.context().unwrap().id(), id);
}
