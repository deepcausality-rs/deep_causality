// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
pub mod augment_data;
pub mod gen_data_time_graph;
pub mod load_data;

use std::error::Error;
use deep_causality::prelude::{Context, Spaceoid, SpaceTempoid, Tempoid, TimeScale};
use crate::types::dateoid::Dataoid;
use crate::types::sampled_date_time_bar::SampledDataBars;
use crate::workflow::gen_data_time_graph::{generate_time_data_context_graph};


pub fn build_time_data_context(
    id: u64,
    name: String,
    data: &SampledDataBars,
    max_time_scale: TimeScale,
)
    -> Result<Context<Dataoid, Spaceoid, Tempoid, SpaceTempoid>, Box<dyn Error>>
{
    let graph = match generate_time_data_context_graph(data, max_time_scale) {
        Ok(g) => g,
        Err(e) => return Err(e),
    };

    Ok(Context::new(id, name,  graph))
}
