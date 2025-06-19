// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{
    CausalityError, Causaloid, Contextuable, ContextuableGraph, NumericalValue,
};
use rust_decimal::prelude::ToPrimitive;

use crate::protocols::rangeable::Rangeable;
use crate::types::alias::{CustomCausaloid, CustomContext};

pub fn get_year_causaloid<'l>(context: &'l CustomContext<'l>) -> CustomCausaloid<'l> {
    let id = 1;
    let description = "Checks if the current price exceeds the all year high";

    // have to add another lifeline to the inner function as you can't pass through a lifeline from an outer scope
    fn contextual_causal_fn<'l>(
        obs: NumericalValue,
        ctx: &'l CustomContext<'l>,
    ) -> Result<bool, CausalityError> {
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }

        // root_index 0
        // year_index 1
        // data_index 2
        let year = ctx.get_node(2).expect("node with index 2 not found");

        let data = year
            .vertex_type()
            .dataoid()
            .expect("Failed to get data out of year node");

        if obs.gt(&data.data_range().close().to_f64().unwrap()) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    Causaloid::new_with_context(id, contextual_causal_fn, Some(context), description)
}
