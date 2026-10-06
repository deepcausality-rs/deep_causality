/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::BaseModelTokio;
use deep_causality::{
    BaseCausaloid, CausalEffect, CausalityError, CausalityErrorEnum, Causaloid,
    IdentificationValue, Model, NumericalValue, PropagatingProcess,
};
use deep_causality_context::{
    BaseContext, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
};
use std::sync::{Arc, RwLock};

/// Node index of the threshold contextoid.
const THRESHOLD: usize = 0;

pub fn build_causal_model() -> Result<BaseModelTokio, ContextIndexError> {
    let id = 1;
    let author = "Marvin Hansen <marvin.hansen@gmail.com>";
    let assumptions = None;
    let context = Arc::new(RwLock::new(get_test_context()?));
    let causaloid = Arc::new(get_test_causaloid(Arc::clone(&context)));
    let description = "This is a test causal model for the Tokio async runtime";

    Ok(Model::new(
        id,
        author,
        description,
        assumptions,
        causaloid,
        Some(context),
    ))
}

pub fn get_test_causaloid(
    context: Arc<RwLock<BaseContext>>,
) -> BaseCausaloid<NumericalValue, bool> {
    let id: IdentificationValue = 1;
    let description = "tests whether data reaches the threshold in the context";

    // Contextual API: fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
    fn causal_fn(
        effect: CausalEffect<NumericalValue>,
        _state: (),
        context: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        let Some(obs) = effect.into_value() else {
            return PropagatingProcess::from_error(CausalityError(
                CausalityErrorEnum::ValueNotAvailable,
            ));
        };

        if obs.is_sign_negative() {
            // Return error via PropagatingProcess
            return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Observation is negative".into(),
            )));
        }

        // Logic can be arbitrary as long as it produces the annotated return type.
        let threshold = match read(context, THRESHOLD) {
            Ok(threshold) => threshold,
            Err(e) => return PropagatingProcess::from_error(e),
        };
        let is_active = obs.ge(&threshold);

        // Return the result wrapped in PropagatingProcess
        PropagatingProcess::pure(is_active)
    }

    Causaloid::new_with_context(id, causal_fn, context, description)
}

/// Reads one `Data` contextoid's payload out of the shared context.
fn read(
    context: Option<Arc<RwLock<BaseContext>>>,
    index: usize,
) -> Result<NumericalValue, CausalityError> {
    let context = context.ok_or(CausalityError(CausalityErrorEnum::MissingContext))?;
    let guard = context.read().map_err(|_| {
        CausalityError(CausalityErrorEnum::Custom(
            "Context lock is poisoned".into(),
        ))
    })?;
    guard
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(Datable::get_data)
        .ok_or_else(|| {
            CausalityError(CausalityErrorEnum::Custom(format!(
                "No Datoid at context index {index}"
            )))
        })
}

/// Builds the model context: one `Data` contextoid holding the activation threshold, on the scale
/// of the observations.
fn get_test_context() -> Result<BaseContext, ContextIndexError> {
    let id = 1;
    let name = "base context for testing";
    let mut context = BaseContext::with_capacity(id, name, 10);

    let threshold = Data::new(id, 0.75);
    context.add_node(Contextoid::new(id, ContextoidType::Datoid(threshold)))?;

    Ok(context)
}
