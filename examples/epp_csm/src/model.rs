/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CsmCausaloid;
use deep_causality::{
    CausalEffectLog, CausalFnOutput, CausalityError, Causaloid, IdentificationValue, NumericalValue,
};

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

pub(crate) fn get_smoke_sensor_causaloid() -> CsmCausaloid {
    let id: IdentificationValue = 1;
    let description = "Tests whether smoke signal exceeds threshold of 65.0";

    fn causal_fn(effect: NumericalValue) -> Result<CausalFnOutput<bool>, CausalityError> {
        let mut log = CausalEffectLog::new();
        verify_obs(effect)?;

        let threshold: NumericalValue = 65.0;
        let is_active = effect.ge(&threshold);

        log.add_entry(&format!(
            "Observation {} is larger than threshold {}: {}",
            effect, threshold, is_active
        ));

        // Log each relevant step
        log.add_entry("Causal function executed successfully");
        // Return the final result and its log.
        Ok(CausalFnOutput::new(is_active, log))
    }

    Causaloid::new(id, causal_fn, description)
}

pub(crate) fn get_fire_sensor_causaloid() -> CsmCausaloid {
    let id: IdentificationValue = 2;
    let description = "Tests if temperature exceeds 85 degree celsius (185 degree Fahrenheit) ";

    fn causal_fn(effect: NumericalValue) -> Result<CausalFnOutput<bool>, CausalityError> {
        let mut log = CausalEffectLog::new();
        verify_obs(effect)?;

        let threshold: NumericalValue = 85.0;
        let is_active = effect.ge(&threshold);

        log.add_entry(&format!(
            "Observation {} is larger than threshold {}: {}",
            effect, threshold, is_active
        ));

        Ok(CausalFnOutput::new(is_active, log))
    }

    Causaloid::new(id, causal_fn, description)
}

pub(crate) fn get_explosion_sensor_causaloid() -> CsmCausaloid {
    let id: IdentificationValue = 3;
    let description =
        "Tests if air pressure exceeds 100 PSI. Regular Atmospheric pressure is 14.696 psi ";

    fn causal_fn(effect: NumericalValue) -> Result<CausalFnOutput<bool>, CausalityError> {
        let mut log = CausalEffectLog::new();
        verify_obs(effect)?;

        let threshold: NumericalValue = 100.0;
        let is_active = effect.ge(&threshold);

        log.add_entry(&format!(
            "Observation {} is larger than threshold {}: {}",
            effect, threshold, is_active
        ));

        Ok(CausalFnOutput::new(is_active, log))
    }
    Causaloid::new(id, causal_fn, description)
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
