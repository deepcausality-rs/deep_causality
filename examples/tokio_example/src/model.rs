/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::BaseModelTokio;
use deep_causality::{
    BaseCausaloid, BaseContext, CausalityError, CausalityErrorEnum, Causaloid, Context, Contextoid,
    ContextoidType, ContextuableGraph, IdentificationValue, Model, NumericalValue,
    PropagatingEffect, Root,
};
use std::sync::{Arc, RwLock};

pub fn build_causal_model() -> BaseModelTokio {
    let id = 1;
    let author = "Marvin Hansen <marvin.hansen@gmail.com>";
    let assumptions = None;
    let causaloid = Arc::new(get_test_causaloid());
    let context = Some(Arc::new(RwLock::new(get_test_context())));
    let description = "This is a test causal model for the Tokio async runtime";

    Model::new(id, author, description, assumptions, causaloid, context)
}

pub fn get_test_causaloid() -> BaseCausaloid<NumericalValue, bool> {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.75";

    // New API: fn(I) -> PropagatingEffect<O>
    fn causal_fn(obs: NumericalValue) -> PropagatingEffect<bool> {
        if obs.is_sign_negative() {
            // Return error via PropagatingEffect
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Observation is negative".into(),
            )));
        }

        // Logic can be arbitrary as long as it produces the annotated return type.
        let threshold: NumericalValue = 0.75;
        let is_active = obs.ge(&threshold);

        // Return the result wrapped in PropagatingEffect
        PropagatingEffect::pure(is_active)
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
    let idx = context
        .add_node(contextoid)
        .expect("failed to add contextoid node");
    assert_eq!(idx, 0);
    assert_eq!(context.size(), 1);

    context
}
