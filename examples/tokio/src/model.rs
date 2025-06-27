/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::errors::CausalityError;
use deep_causality::prelude::{
    BaseCausaloid, BaseContext, BaseModel, Causaloid, Context, Contextoid, ContextoidType,
    ContextuableGraph, IdentificationValue, Model, NumericalValue, Root,
};
use std::sync::Arc;

pub fn build_causal_model() -> BaseModel {
    let id = 1;
    let author = "Marvin Hansen <marvin.hansen@gmail.com>";
    let assumptions = None;
    let causaloid = Arc::new(get_test_causaloid());
    let context = Some(Arc::new(get_test_context()));
    let description = "This is a test causal model";

    Model::new(id, author, description, assumptions, causaloid, context)
}

pub fn get_test_causaloid() -> BaseCausaloid {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.75";
    fn causal_fn(obs: &NumericalValue) -> Result<bool, CausalityError> {
        if obs.is_sign_negative() {
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.75;
        if !obs.ge(&threshold) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    Causaloid::new(id, causal_fn, description)
}

fn get_test_context() -> BaseContext {
    let id = 1;
    let name = "base context for testing";
    let mut context = Context::with_capacity(id, name, 10);
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    let idx = context.add_node(contextoid);
    assert_eq!(idx, 0);
    assert_eq!(context.size(), 1);

    context
}
