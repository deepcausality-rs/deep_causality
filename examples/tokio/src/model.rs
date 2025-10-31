/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    BaseCausaloid, BaseContext, BaseModel, CausalityError, Causaloid, Context, Contextoid,
    ContextoidType, ContextuableGraph, IdentificationValue, Model, NumericalValue,
    PropagatingEffect, Root,
};
use std::sync::{Arc, RwLock};

pub fn build_causal_model() -> BaseModel {
    let id = 1;
    let author = "Marvin Hansen <marvin.hansen@gmail.com>";
    let assumptions = None;
    let causaloid = Arc::new(get_test_causaloid());
    let context = Some(Arc::new(RwLock::new(get_test_context())));
    let description = "This is a test causal model";

    Model::new(id, author, description, assumptions, causaloid, context)
}

pub fn get_test_causaloid() -> BaseCausaloid {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.75";

    // This function must now match the standard `CausalFn` signature.
    fn causal_fn(evidence: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        // Safely extract the numerical value from the generic Evidence enum.
        let obs = match evidence {
            PropagatingEffect::Numerical(val) => *val,
            _ => return Err(CausalityError("Expected Numerical evidence.".into())),
        };

        if obs.is_sign_negative() {
            return Err(CausalityError("Observation is negative".into()));
        }

        let threshold: NumericalValue = 0.75;
        let is_active = obs.ge(&threshold);

        // Return the boolean result wrapped in the standard PropagatingEffect enum.
        Ok(PropagatingEffect::Deterministic(is_active))
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
