// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::model::causaloid_year::get_year_causaloid;
use deep_causality::prelude::{CausableGraph, Causaloid, CausaloidGraph, Context, Spaceoid, SpaceTempoid, Tempoid};
use crate::model::causaloid_month::get_month_causaloid;
use crate::types::dateoid::Dataoid;

pub mod utils;
pub mod causaloid_year;
pub mod causaloid_month;

type CustomContext<'l> =  Context<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>;
type CustomCausaloid<'l> = Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>;

pub fn get_main_causaloid<'l>(
    context: &'l CustomContext<'l>
)
    -> CustomCausaloid<'l>
{
    build_causaloid(context)
}

fn build_causaloid<'l>(
    context: &'l CustomContext<'l>
)
    -> CustomCausaloid<'l>{
    let mut g = CausaloidGraph::new();

    // Add the root causaloid to the causaloid graph
    let root_causaloid =  get_year_causaloid(context);
    let root_index = g.add_root_causaloid(root_causaloid);

    // Add the month causaloid to the causaloid graph
    let month_causaloid =  get_month_causaloid(context);
    let month_index = g.add_causaloid(month_causaloid);

    let _ = g.add_edge(root_index, month_index);

    // Here we wrap the causal graph into a causaloid
    Causaloid::from_causal_graph_with_context(
        0,
        g,
        Option::from(context),
            "Causaloid main graph"
    )

}