/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CsmCausaloid;
use deep_causality::{
    CausalEffect, CausalityError, CausalityErrorEnum, Causaloid, IdentificationValue,
    NumericalValue, PropagatingProcess,
};
use deep_causality_context::{
    BaseContext, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph,
    Data,
};
use std::sync::{Arc, RwLock};

/// Contextoid id: smoke alarm threshold, smoke signal level (the example states no unit).
const SMOKE_THRESHOLD: ContextoidId = 1;
/// Contextoid id: fire alarm threshold, degree Celsius.
const FIRE_THRESHOLD: ContextoidId = 2;
/// Contextoid id: explosion alarm threshold, air pressure in psi.
const EXPLOSION_THRESHOLD: ContextoidId = 3;

pub(crate) fn get_smoke_sensor_data() -> [NumericalValue; 12] {
    [
        10.0, 8.0, 3.4, 7.0, 12.1, 30.89, 45.3, 60.89, 78.23, 89.8, 88.7, 91.3,
    ]
}

pub(crate) fn get_fire_sensor_data() -> [NumericalValue; 12] {
    [
        20.0, 21.0, 23.4, 22.0, 22.1, 33.89, 54.3, 60.89, 78.23, 89.8, 95.7, 99.3,
    ]
}

pub(crate) fn get_explosion_sensor_data() -> [NumericalValue; 12] {
    [
        14.6, 14.6, 14.6, 222.0, 270.1, 90.89, 54.3, 29.89, 14.6, 14.6, 14.6, 14.6,
    ]
}

/// Builds the sensor context: one `Data` contextoid per alarm threshold.
///
/// - smoke: smoke signal level, 65.0 (the example states no unit);
/// - fire: temperature, 85.0 degree Celsius (185 degree Fahrenheit);
/// - explosion: air pressure, 100.0 psi (regular atmospheric pressure is 14.696 psi).
pub(crate) fn get_sensor_context() -> Result<BaseContext, ContextIndexError> {
    let facts = [
        (SMOKE_THRESHOLD, 65.0),
        (FIRE_THRESHOLD, 85.0),
        (EXPLOSION_THRESHOLD, 100.0),
    ];
    let mut context = BaseContext::with_capacity(1, "Sensor thresholds", facts.len());
    for (id, threshold) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, threshold)),
        ))?;
    }
    Ok(context)
}

pub(crate) fn get_smoke_sensor_causaloid(context: Arc<RwLock<BaseContext>>) -> CsmCausaloid {
    let id: IdentificationValue = 1;
    let description = "Tests whether the smoke signal reaches the smoke threshold in the context";

    fn causal_fn(
        effect: CausalEffect<NumericalValue>,
        _state: (),
        context: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        reaches_threshold(effect, context, SMOKE_THRESHOLD)
    }

    Causaloid::new_with_context(id, causal_fn, context, description)
}

pub(crate) fn get_fire_sensor_causaloid(context: Arc<RwLock<BaseContext>>) -> CsmCausaloid {
    let id: IdentificationValue = 2;
    let description =
        "Tests whether the temperature (degree Celsius) reaches the fire threshold in the context";

    fn causal_fn(
        effect: CausalEffect<NumericalValue>,
        _state: (),
        context: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        reaches_threshold(effect, context, FIRE_THRESHOLD)
    }

    Causaloid::new_with_context(id, causal_fn, context, description)
}

pub(crate) fn get_explosion_sensor_causaloid(context: Arc<RwLock<BaseContext>>) -> CsmCausaloid {
    let id: IdentificationValue = 3;
    let description = "Tests whether air pressure (psi) reaches the explosion threshold in the context. Regular atmospheric pressure is 14.696 psi";

    fn causal_fn(
        effect: CausalEffect<NumericalValue>,
        _state: (),
        context: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        reaches_threshold(effect, context, EXPLOSION_THRESHOLD)
    }

    Causaloid::new_with_context(id, causal_fn, context, description)
}

/// Verifies the observation, reads the threshold with contextoid id `threshold_id` from the
/// context, and returns whether the observation reaches it.
fn reaches_threshold(
    effect: CausalEffect<NumericalValue>,
    context: Option<Arc<RwLock<BaseContext>>>,
    threshold_id: ContextoidId,
) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
    let Some(obs) = effect.into_value() else {
        return PropagatingProcess::from_error(CausalityError(
            CausalityErrorEnum::ValueNotAvailable,
        ));
    };
    if let Err(e) = verify_obs(obs) {
        return PropagatingProcess::from_error(e);
    }

    match read_threshold(context, threshold_id) {
        Ok(threshold) => PropagatingProcess::pure(obs.ge(&threshold)),
        Err(e) => PropagatingProcess::from_error(e),
    }
}

/// Reads the threshold with contextoid id `id` out of the shared sensor context.
fn read_threshold(
    context: Option<Arc<RwLock<BaseContext>>>,
    id: ContextoidId,
) -> Result<NumericalValue, CausalityError> {
    let context = context.ok_or(CausalityError(CausalityErrorEnum::MissingContext))?;
    let guard = context.read().map_err(|_| {
        CausalityError(CausalityErrorEnum::Custom(
            "Sensor context lock is poisoned".into(),
        ))
    })?;
    guard.get_data_by_id(id).ok_or_else(|| {
        CausalityError(CausalityErrorEnum::Custom(format!(
            "No threshold Datoid with contextoid id {id}"
        )))
    })
}

fn verify_obs(obs: NumericalValue) -> Result<(), CausalityError> {
    if obs.is_nan() {
        return Err(CausalityError(CausalityErrorEnum::Custom(
            "Observation is NULL/NAN".into(),
        )));
    }

    if obs.is_infinite() {
        return Err(CausalityError(CausalityErrorEnum::Custom(
            "Observation is infinite".into(),
        )));
    }

    if obs.is_sign_negative() {
        return Err(CausalityError(CausalityErrorEnum::Custom(
            "Observation is negative".into(),
        )));
    }

    Ok(())
}
