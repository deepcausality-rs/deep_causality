// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{CausalityError, Causaloid, Context, Contextuable, ContextuableGraph, NumericalValue, Spaceoid, SpaceTempoid, Tempoid};
use rust_decimal::prelude::ToPrimitive;
use crate::protocols::rangeable::Rangeable;
use crate::types::dateoid::Dataoid;

pub fn get_year_causaloid<'l>(
    context: &'l Context<Dataoid, Spaceoid, Tempoid, SpaceTempoid>
)
    -> Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>
{
    let id = 1;
    let description = "Checks if the current price exceeds the all year high";

    fn contextual_causal_fn(
        obs: NumericalValue,
        ctx: &Context<Dataoid, Spaceoid, Tempoid, SpaceTempoid>,
    )
        -> Result<bool, CausalityError>
    {
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }

        // root_index 0
        // year_index 1
        // data_index 2
        let year = ctx.get_node(2)
            .expect("node with index 2 not found");

        let data = year.vertex_type().dataoid()
            .expect("Failed to get data out of year node");

        if obs.gt(&data.data_range().close().to_f64().unwrap()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    Causaloid::new_with_context(
        id,
        contextual_causal_fn,
        Some(context),
        description
    )
}