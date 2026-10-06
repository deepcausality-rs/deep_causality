/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{
    CausalAction, CausalEffect, CausalityError, CausalityErrorEnum, Causaloid, IdentificationValue,
    NumericalValue, PropagatingProcess,
};
use deep_causality_context::{
    BaseContext, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph,
    Data,
};
use deep_causality_ethos::{DeonticError, EffectEthos, TeloidModal};

use std::sync::{Arc, RwLock};

/// Contextoid id: temperature alert threshold, on the scale of the temperature reading.
const ALERT_THRESHOLD: ContextoidId = 1;

/// The scalar this example works in.
pub type FloatType = f64;

// Type aliases for manageable generics
pub type CsmCausaloid = Causaloid<f64, bool, (), Arc<RwLock<BaseContext>>>;

pub type CsmEthos = EffectEthos<
    deep_causality_context::Data<NumericalValue>,
    deep_causality_context::EuclideanSpace<FloatType>,
    deep_causality_context::NewtonianTime<FloatType>,
    deep_causality_context::NewtonianSpacetime<FloatType>,
>;

pub(crate) fn get_effect_ethos() -> Result<CsmEthos, DeonticError> {
    let mut ethos = EffectEthos::new().add_deterministic_norm(
        1,
        "high_temp_alert",
        &["temperature"],
        |_context, _action| true, // Always active for demo
        TeloidModal::Impermissible,
        1,
        1,
        1,
    )?;
    ethos.verify_graph()?;
    Ok(ethos)
}

pub(crate) fn get_test_causaloid(context: Arc<RwLock<BaseContext>>) -> CsmCausaloid {
    let id: IdentificationValue = 1;
    let description = "tests whether data reaches the alert threshold in the context";

    // New API: fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
    fn context_causal_fn(
        effect: CausalEffect<f64>,
        _state: (),
        context: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        let Some(obs) = effect.into_value() else {
            return PropagatingProcess::from_error(CausalityError(
                CausalityErrorEnum::ValueNotAvailable,
            ));
        };

        if obs.is_sign_negative() {
            return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Observation is negative".into(),
            )));
        }

        let threshold = match read_threshold(context, ALERT_THRESHOLD) {
            Ok(threshold) => threshold,
            Err(e) => return PropagatingProcess::from_error(e),
        };
        let is_active = obs.ge(&threshold);

        PropagatingProcess::pure(is_active)
    }

    Causaloid::new_with_context(id, context_causal_fn, context, description)
}

/// Reads the threshold with contextoid id `id` out of the shared context.
fn read_threshold(
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
            "No threshold Datoid with contextoid id {id}"
        )))
    })
}

pub(crate) fn get_alert_action() -> CausalAction {
    let func = || {
        println!("Alert! High temperature detected!");
        Ok(())
    };
    let descr = "Action that triggers an alert";
    let version = 1;
    CausalAction::new(func, descr, version)
}

/// Builds the shared context: one `Data` contextoid holding the temperature alert threshold, on
/// the scale of the temperature reading.
pub(crate) fn get_base_context() -> Result<BaseContext, ContextIndexError> {
    let facts = [(ALERT_THRESHOLD, 0.55)];
    let mut context = BaseContext::with_capacity(1, "base context", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }

    Ok(context)
}
