/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::BaseModelTokio;
use deep_causality::{
    BaseCausaloid, BaseContext, CausalEffectLog, CausalFnOutput, CausalityError, Causaloid,
    Context, Contextoid, ContextoidType, ContextuableGraph, IdentificationValue, Model,
    NumericalValue, Root,
};
use std::sync::{Arc, RwLock};

pub fn build_causal_model() -> BaseModelTokio {
    let id = 1;
    let author = "Marvin Hansen <marvin.hansen@gmail.com>";
    let assumptions = None;
    let causaloid = Arc::new(get_test_causaloid());
    let context = Some(Arc::new(RwLock::new(get_test_context())));
    let description = "This is a test causal model for the Tokio asyn runtime";

    Model::new(id, author, description, assumptions, causaloid, context)
}

pub fn get_test_causaloid() -> BaseCausaloid<NumericalValue, bool> {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.75";

    // The signature: CausalFn<I: IntoEffectValue, O: IntoEffectValue> = fn(value: I) -> Result<CausalFnOutput<O>, CausalityError>
    // IntoEffectValue is implemented by default for all primitive types and by all complex types supported
    // by PropagatingEffect. Notice, when you call causaloid.evaluate(&PropagatingEffect), the PropagatingEffect
    // converts automatically into the matching NumericalValue via the IntoEffectValue default implementation.
    fn causal_fn(obs: NumericalValue) -> Result<CausalFnOutput<bool>, CausalityError> {
        // the log is part of the CausalFnOutput.
        // When multiple causaloid are called in sequence, the logs are appended to the resulting
        // propagating effect meaning the final result carries a full immutable history how it was produced.
        let mut log = CausalEffectLog::new();
        if obs.is_sign_negative() {
            // At any point, you can short circuit and return an error,
            return Err(CausalityError("Observation is negative".into()));
        }

        // Logic can be arbitrary as long as it produces the annotated return type.
        let threshold: NumericalValue = 0.75;
        let is_active = obs.ge(&threshold);
        log.add_entry(&format!(
            "Observation {} is larger than threshold {}: {}",
            obs, threshold, is_active
        ));

        // Log each relevant step
        log.add_entry("Ccausal function executed successfully");
        // Return the final result and its log.
        Ok(CausalFnOutput::new(is_active, log))
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
