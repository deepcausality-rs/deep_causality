// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::*;
use deep_causality::types::alias_types::{DescriptionValue, IdentificationValue};
use deep_causality::utils::bench_utils_graph;
use deep_causality::utils::test_utils;
use deep_causality::utils::test_utils::{get_inferable_coll, get_test_causality_vec};

#[test]
fn test_build_causaloid() {
    let id: IdentificationValue = 1;
    let description: String = "tests whether data exceeds threshold of 0.55".to_string() as DescriptionValue;
    let data_set_id = "Test data".to_string() as DescriptionValue;
    let inferable_coll = get_inferable_coll(false);
    let inverse_inferable_coll = get_inferable_coll(true);
    fn causal_fn(_obs: NumericalValue) -> Result<bool, CausalityError> { Ok(true) }

    let causaloid = build_causaloid(
        id,
        causal_fn,
        description,
        data_set_id,
        &inferable_coll,
        &inverse_inferable_coll,
    ).unwrap();

    assert!(causaloid.is_singleton());

    assert_eq!(01, causaloid.id());
    assert_eq!("tests whether data exceeds threshold of 0.55".to_string(), causaloid.description());
    assert!(!causaloid.is_active());
    assert!(causaloid.explain().is_err());
}

#[test]
fn test_build_causaloid_err() {
    let id: IdentificationValue = 1;
    let description: String = "".to_string() as DescriptionValue;
    let data_set_id = "".to_string() as DescriptionValue;
    let inferable_coll = get_inferable_coll(false);
    let inverse_inferable_coll = get_inferable_coll(true);
    fn causal_fn(_obs: NumericalValue) -> Result<bool, CausalityError> { Ok(true) }

    let causaloid = build_causaloid(id, causal_fn, description, data_set_id, &inferable_coll, &inverse_inferable_coll,);
    assert!(causaloid.is_err());
}

#[test]
fn test_build_causaloid_from_vec() {
    let id: IdentificationValue = 1;
    let description: String = "tests whether data exceeds threshold of 0.55".to_string() as DescriptionValue;
    let data_set_id = "Test data".to_string() as DescriptionValue;
    let causal_vec = get_test_causality_vec();

    let causaloid = build_causaloid_from_vec(id, causal_vec, data_set_id, description,).unwrap();

    assert!(!causaloid.is_singleton());
    assert_eq!(01, causaloid.id());
    assert_eq!("tests whether data exceeds threshold of 0.55".to_string(), causaloid.description());
    assert!(!causaloid.is_active());
    assert!(causaloid.explain().is_err());
}

#[test]
fn test_build_causaloid_from_vec_err() {
    let id: IdentificationValue = 1;
    let description: String = "tests whether data exceeds threshold of 0.55".to_string() as DescriptionValue;
    let data_set_id = "".to_string() as DescriptionValue;
    let causal_vec = get_test_causality_vec();

    let causaloid = build_causaloid_from_vec(id, causal_vec, data_set_id, description,);
    assert!(causaloid.is_err());
}

#[test]
fn test_build_causaloid_from_graph() {
    let id: IdentificationValue = 1;
    let description: String = "tests whether data exceeds threshold of 0.55".to_string() as DescriptionValue;
    let data_set_id = "Test data".to_string() as DescriptionValue;
    let (causal_graph, _) = bench_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = build_causaloid_from_graph(id, causal_graph, data_set_id, description,).unwrap();

    assert!(!causaloid.is_singleton());
    assert_eq!(01, causaloid.id());
    assert_eq!("tests whether data exceeds threshold of 0.55".to_string(), causaloid.description());
    assert!(!causaloid.is_active());
    assert!(causaloid.explain().is_err());
}

#[test]
fn test_build_causaloid_from_graph_descr_err() {
    let id: IdentificationValue = 1;
    let description: String = "".to_string() as DescriptionValue;
    let data_set_id = "Test data".to_string() as DescriptionValue;
    let (causal_graph, _) = bench_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = build_causaloid_from_graph(id, causal_graph, data_set_id, description,);

    assert!(causaloid.is_err());
}

#[test]
fn test_build_causaloid_from_graph_data_err() {
    let id: IdentificationValue = 1;
    let description: String = "tests whether data exceeds threshold of 0.55".to_string() as DescriptionValue;
    let data_set_id = "".to_string() as DescriptionValue;
    let (causal_graph, _) = bench_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = build_causaloid_from_graph(id, causal_graph, data_set_id, description,);

    assert!(causaloid.is_err());
}

#[test]
fn test_from_causal_collection() {
    let id: IdentificationValue = 01;
    let description: String = "tests whether data exceeds threshold of 0.55".to_string() as DescriptionValue;
    let data_set_id = "Test data".to_string() as DescriptionValue;
    let causal_coll = test_utils::get_test_causality_vec();

    let data = [0.89, 0.89, 0.99];
    assert_eq!(data.len(), causal_coll.len());

    let causaloid = Causaloid::from_causal_collection(id, causal_coll, data_set_id, description);
    assert!(!causaloid.is_singleton());

    assert!(!causaloid.is_active());
    assert!(causaloid.explain().is_err());

    let res = causaloid.verify_all_causes(&data, None);
    assert!(res.is_ok());

    assert!(res.unwrap());
    assert!(causaloid.is_active());
}

#[test]
fn test_from_causal_graph() {
    let id: IdentificationValue = 01;
    let description: String = "tests whether data exceeds threshold of 0.55".to_string() as DescriptionValue;
    let data_set_id = "Test data".to_string() as DescriptionValue;
    let (causal_graph, data) = bench_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = Causaloid::from_causal_graph(id, causal_graph, data_set_id, description);
    assert!(!causaloid.is_singleton());

    assert!(!causaloid.is_active());
    assert!(causaloid.explain().is_err());

    let res = causaloid.verify_all_causes(&data, None);
    assert!(res.is_ok());

    assert!(res.unwrap());
    assert!(causaloid.is_active());
}

#[test]
fn test_causal_graph()
{
    let id: IdentificationValue = 01;
    let description: String = "tests whether data exceeds threshold of 0.55".to_string() as DescriptionValue;
    let data_set_id = "Test data".to_string() as DescriptionValue;
    let (causal_graph, _) = bench_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = Causaloid::from_causal_graph(id, causal_graph, data_set_id, description);
    assert!(!causaloid.is_singleton());

    assert!(causaloid.causal_graph().is_some());
    assert!(causaloid.causal_collection().is_none());
}

#[test]
fn test_causal_collection() {
    let id: IdentificationValue = 01;
    let description: String = "tests whether data exceeds threshold of 0.55".to_string() as DescriptionValue;
    let data_set_id = "Test data".to_string() as DescriptionValue;
    let causal_coll = test_utils::get_test_causality_vec();

    let data = [0.89, 0.89, 0.99];
    assert_eq!(data.len(), causal_coll.len());

    let causaloid = Causaloid::from_causal_collection(id, causal_coll, data_set_id, description);
    assert!(!causaloid.is_singleton());

    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
}

#[test]
fn test_explain() {
    let causaloid = test_utils::get_test_causaloid();
    assert!(!causaloid.is_active());

    // We expect and error because the causaloid has not yet been activated.
    let actual = causaloid.explain();
    assert!(actual.is_err());

    let obs: f64 = 0.78;
    let res = causaloid.verify_single_cause(&obs).unwrap();
    assert!(res);
    assert!(causaloid.is_active());

    let actual = causaloid.explain().unwrap();
    let expected = "Causaloid: 1 tests whether data exceeds threshold of 0.55 on last data 0.78 evaluated to true".to_string();
    assert_eq!(actual, expected);
}

#[test]
fn test_verify_single_cause() {
    let causaloid = test_utils::get_test_causaloid();
    assert!(!causaloid.is_active());

    let obs: f64 = 0.78;
    let res = causaloid.verify_single_cause(&obs).unwrap();
    assert!(res);
    assert!(causaloid.is_active());
}

#[test]
fn test_to_string() {
    let causaloid = test_utils::get_test_causaloid();
    assert!(!causaloid.is_active());

    let expected = format!("Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55 is active: false");
    let actual = causaloid.to_string();

    assert_eq!(actual, expected);
}