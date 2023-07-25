// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use deep_causality::prelude::{Identifiable, Model};
use deep_causality::utils::test_utils::get_test_causaloid;

#[test]
fn test_new() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = &None;
    let causaloid = &get_test_causaloid();

    let model = Model::new(id, author, description, assumptions, causaloid);

    assert_eq!(model.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = &None;
    let causaloid = &get_test_causaloid();

    let model = Model::new(id, author, description, assumptions, causaloid);

    assert_eq!(model.id(), id);
}

#[test]
fn test_author() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = &None;
    let causaloid = &get_test_causaloid();

    let model = Model::new(id, author, description, assumptions, causaloid);

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
}

#[test]
fn test_description() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = &None;
    let causaloid = &get_test_causaloid();

    let model = Model::new(id, author, description, assumptions, causaloid);

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
}

#[test]
fn test_assumptions() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = &None;
    let causaloid = &get_test_causaloid();

    let model = Model::new(id, author, description, assumptions, causaloid);

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_none());
}

#[test]
fn test_causaloid() {
    let id = 1;
    let author = "John Doe";
    let description = "This is a test model";
    let assumptions = &None;
    let causaloid = &get_test_causaloid();

    let model = Model::new(id, author, description, assumptions, causaloid);

    assert_eq!(model.id(), id);
    assert_eq!(model.author(), author);
    assert_eq!(model.description(), description);
    assert!(model.assumptions().is_none());
    assert_eq!(model.causaloid(), causaloid);
}