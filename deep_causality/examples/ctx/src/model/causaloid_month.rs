// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{CausalityError, Causaloid, Context, Contextuable, ContextuableGraph, NumericalValue, Spaceoid, SpaceTempoid, Tempoid};
use rust_decimal::prelude::ToPrimitive;
use crate::protocols::rangeable::Rangeable;
use crate::types::dateoid::Dataoid;

pub fn get_month_causaloid<'l>(
    context: &'l Context<Dataoid, Spaceoid, Tempoid, SpaceTempoid>
)
    -> Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>
{
    let id = 2;
    let description = "Checks if the current price exceeds the monthly high level";

    // The causal function is a function that takes the current price and returns a boolean
    // that indicates whether the current price exceeds the monthly high level.
    // The cause being some fabricated nonsense metrics i.e. price above monthly high and the effect
    // being a monthly breakout.

    // The causal fucntion must be a function and not a closure because the function
    // will be coercived into a function pointer later on, which is not possible with a closure.
    // Within the causal function, you can write safety as many closures as you want. See below.
    fn contextual_causal_fn(
        obs: NumericalValue,
        ctx: &Context<Dataoid, Spaceoid, Tempoid, SpaceTempoid>,
    )
        -> Result<bool, CausalityError>
    {
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }

        // We just pick a random month here to use as an example.
        // In practice, you might want to use a dynamic secondary index
        // to determine the actual index of the previous or current month tempoid relative
        // to the now() timestamp. To do this, you may extend the context
        // with an extension trait and corresponding implementation.
        // See http://xion.io/post/code/rust-extension-traits.html
        let month = ctx.get_node(14)
            .expect("node with index 2 not found");
        let data = month.vertex_type().dataoid()
            .expect("Failed to get data out of year node");

        let check_month_breakout = || {
            // This logic is obviously complete nonsense, but it demonstrates that you can
            // split complex causal functions into multiple closures.
            data.data_range().close_above_open() && !data.data_range().close_below_open()
        };

        // Another closure that captures the context within the causal function.
        let check_price_above_high = || {
            obs.gt(&data.data_range().high().to_f64().unwrap())
        };

        // With the closures in place, the main logic becomes straightforward and simple to understand.
        if check_price_above_high() && check_month_breakout()
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    Causaloid::new_with_context(
        id,
        contextual_causal_fn,
        Some(context),
        description,
    )
}