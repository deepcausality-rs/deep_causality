/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseCausaloid, CausalityError, Causaloid, IdentificationValue, NumericalValue,
    PropagatingEffect,
};

pub fn get_smoke_sensor_causaloid() -> BaseCausaloid {
    let id: IdentificationValue = 1;
    let description = "Tests whether smoke signal exceeds threshold of 65.0";

    fn causal_fn(effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(effect)?;
        verify_obs(obs)?;

        let threshold: NumericalValue = 65.0;
        let is_active = obs.ge(&threshold);

        Ok(PropagatingEffect::Deterministic(is_active))
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn get_fire_sensor_causaloid() -> BaseCausaloid {
    let id: IdentificationValue = 2;
    let description = "Tests if temperature exceeds 85 degree celsius (185 degree Fahrenheit) ";

    fn causal_fn(effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(effect)?;
        verify_obs(obs)?;

        let threshold: NumericalValue = 85.0;
        let is_active = obs.ge(&threshold);

        Ok(PropagatingEffect::Deterministic(is_active))
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn get_explosion_sensor_causaloid() -> BaseCausaloid {
    let id: IdentificationValue = 3;
    let description =
        "Tests if air pressure exceeds 100 PSI. Regular Atmospheric pressure is 14.696 psi ";

    fn causal_fn(effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(effect)?;
        verify_obs(obs)?;

        let threshold: NumericalValue = 100.0;
        let is_active = obs.ge(&threshold);

        Ok(PropagatingEffect::Deterministic(is_active))
    }

    Causaloid::new(id, causal_fn, description)
}

// Helper to reduce code duplication in causal functions.
fn unpack_evidence(effect: &PropagatingEffect) -> Result<NumericalValue, CausalityError> {
    match effect {
        PropagatingEffect::Numerical(val) => Ok(*val),
        _ => Err(CausalityError("Expected Numerical effect.".into())),
    }
}

fn verify_obs(obs: NumericalValue) -> Result<(), CausalityError> {
    if obs.is_nan() {
        return Err(CausalityError("Observation is NULL/NAN".into()));
    }

    if obs.is_infinite() {
        return Err(CausalityError("Observation is infinite".into()));
    }

    if obs.is_sign_negative() {
        return Err(CausalityError("Observation is negative".into()));
    }

    Ok(())
}
