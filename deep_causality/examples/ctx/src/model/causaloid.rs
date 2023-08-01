// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{CausalityError, Causaloid, Context, IdentificationValue, NumericalValue, Spaceoid, SpaceTempoid, Tempoid};
use crate::types::dateoid::Dataoid;

pub fn get_first_causaloid<'l>(
    context: &'l Context<Dataoid, Spaceoid, Tempoid, SpaceTempoid>
)
    -> Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>
{

    let id: IdentificationValue = 2;
    let description = " ";

    fn causal_fn(obs: NumericalValue) -> Result<bool, CausalityError>
    {
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }


        Ok(true)
    }

    Causaloid::new_with_context(id, causal_fn, Some(context), description)
}