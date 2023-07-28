// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use deep_causality::prelude::{CausalityError, Causaloid, IdentificationValue, NumericalValue};

pub fn get_smoke_sensor_causaloid()
    ->  Causaloid<'static>
{
    let id: IdentificationValue = 1;
    let description = "Tests whether smoke signal exceeds threshold of 65.0";
    fn causal_fn(obs: NumericalValue) -> Result<bool, CausalityError> {
        if verify_obs(obs).is_err() {
            return Err(verify_obs(obs).err().unwrap());
        }

        let threshold: NumericalValue = 65.0;
        if !obs.ge(&threshold) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn get_fire_sensor_causaloid()
    ->  Causaloid<'static>
{
    let id: IdentificationValue = 2;
    let description = "Tests if temperature exceeds 85 degree celsius (185 degree Fahrenheit) ";
    fn causal_fn(obs: NumericalValue) -> Result<bool, CausalityError> {
        if verify_obs(obs).is_err() {
            return Err(verify_obs(obs).err().unwrap());
        }

        let threshold: NumericalValue = 85.0;
        if !obs.ge(&threshold) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    Causaloid::new(id, causal_fn, description)
}

pub fn get_explosion_sensor_causaloid()
    ->  Causaloid<'static>
{
    let id: IdentificationValue = 3;
    let description = "Tests if air pressure exceeds 100 PSI. Regular Atmospheric pressure is 14.696 psi ";

    fn causal_fn(obs: NumericalValue) -> Result<bool, CausalityError> {
        if verify_obs(obs).is_err() {
            return Err(verify_obs(obs).err().unwrap());
        }

        let threshold: NumericalValue = 100.0;
        if !obs.ge(&threshold) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    Causaloid::new(id, causal_fn, description)
}

fn verify_obs(
    obs: NumericalValue
)
    -> Result<(), CausalityError>
{
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