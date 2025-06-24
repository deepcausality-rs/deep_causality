// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::*;
use std::sync::Arc;

use crate::utils::*;

// BaseContext is a type alias for a basic context that can be used for testing
// It matches the type signature of the base causaloid also uses in these tests.
// See src/types/alias_types/csm_types for definition.
fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    let mut context = Context::with_capacity(id, name, 10);
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    let idx = context.add_node(contextoid);
    assert_eq!(idx, 0);
    assert_eq!(context.size(), 1);

    context
}

#[test]
fn test_new() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    fn causal_fn(obs: &NumericalValue) -> Result<bool, CausalityError> {
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }
        let threshold: NumericalValue = 0.75;
        if !obs.ge(&threshold) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    let causaloid: BaseCausaloid = Causaloid::new(id, causal_fn, description);

    assert!(causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_none());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_none());
}

#[test]
fn test_new_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let context = get_context();

    fn contextual_causal_fn(
        obs: &NumericalValue,
        ctx: &Arc<BaseContext>,
    ) -> Result<bool, CausalityError> {
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }

        // get contextoid by ID
        let contextoid = ctx.get_node(0).expect("Could not find contextoid");

        // extract data from the contextoid
        let val = contextoid.id() as f64;

        // run any arithmetic with the data from the contextois
        if val == 1.0 {
            Ok(true)
        } else {
            // relate the observation (obs) to the data (val) from the contextoid
            if !obs.ge(&val) {
                Ok(false)
            } else {
                Ok(true)
            }
        }
    }

    let causaloid: BaseCausaloid =
        Causaloid::new_with_context(id, contextual_causal_fn, Arc::new(context), description);

    assert!(causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_none());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_some());
}

#[test]
fn test_from_causal_collection() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_coll = test_utils::get_test_causality_vec();

    let data = [0.89, 0.89, 0.99];
    assert_eq!(data.len(), causal_coll.len());

    let causaloid = Causaloid::from_causal_collection(id, Arc::new(causal_coll), description);
    assert!(!causaloid.is_singleton());

    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_none());
}

#[test]
fn test_from_causal_collection_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_coll = test_utils::get_test_causality_vec();
    let context = get_context();

    let data = [0.89, 0.89, 0.99];
    assert_eq!(data.len(), causal_coll.len());

    let causaloid = Causaloid::from_causal_collection_with_context(
        id,
        Arc::new(causal_coll),
        Arc::new(context),
        description,
    );

    assert!(!causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_some());
}

#[test]
fn test_from_causal_graph() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let (causal_graph, data) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = Causaloid::from_causal_graph(id, Arc::new(causal_graph), description);
    assert!(!causaloid.is_singleton());

    assert!(!causaloid.is_active());
    assert!(causaloid.explain().is_err());

    let res = causaloid.verify_all_causes(&data, None);
    assert!(res.is_ok());

    assert!(res.unwrap());
    assert!(causaloid.is_active());
    assert!(causaloid.context().is_none());
}

#[test]
fn test_from_causal_graph_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let (causal_graph, data) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();
    let context = get_context();

    let causaloid = Causaloid::from_causal_graph_with_context(
        id,
        Arc::new(causal_graph),
        Arc::new(context),
        description,
    );
    assert!(!causaloid.is_singleton());

    assert!(!causaloid.is_active());
    assert!(causaloid.explain().is_err());

    let res = causaloid.verify_all_causes(&data, None);
    assert!(res.is_ok());

    assert!(res.unwrap());
    assert!(causaloid.is_active());
    assert!(causaloid.context().is_some());
}

#[test]
fn test_causal_graph() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let (causal_graph, _) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = Causaloid::from_causal_graph(id, Arc::new(causal_graph), description);
    assert!(!causaloid.is_singleton());

    assert!(causaloid.causal_graph().is_some());
    assert!(causaloid.causal_collection().is_none());
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
    let expected =
        "Causaloid: 1 tests whether data exceeds threshold of 0.55 evaluated to true".to_string();
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

    let expected = "Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55 is active: false has context: false".to_string();
    let actual = causaloid.to_string();

    assert_eq!(actual, expected);
}
