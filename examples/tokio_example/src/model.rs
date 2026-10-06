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
    BaseContext, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph,
    Data,
};
use std::sync::{Arc, RwLock};

/// Contextoid id: activation threshold, on the scale of the observations.
const THRESHOLD: ContextoidId = 1;

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

/// Reads the `Data` contextoid with contextoid id `id` out of the shared context.
fn read(
    context: Option<Arc<RwLock<BaseContext>>>,
    id: ContextoidId,
) -> Result<NumericalValue, CausalityError> {
    let context = context.ok_or(CausalityError(CausalityErrorEnum::MissingContext))?;
    let guard = context.read().map_err(|_| {
        CausalityError(CausalityErrorEnum::Custom(
            "Context lock is poisoned".into(),
        ))
    })?;
    guard.get_data_by_id(id).ok_or_else(|| {
        CausalityError(CausalityErrorEnum::Custom(format!(
            "No Datoid with contextoid id {id}"
        )))
    })
}

/// Builds the model context: one `Data` contextoid holding the activation threshold, on the scale
/// of the observations.
fn get_test_context() -> Result<BaseContext, ContextIndexError> {
    let facts = [(THRESHOLD, 0.75)];
    let mut context = BaseContext::with_capacity(1, "base context for testing", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }

    Ok(context)
}
