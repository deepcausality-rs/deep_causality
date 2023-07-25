// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use std::error::Error;
use deep_causality::prelude::{ContextMatrixGraph, Spaceoid, Tempoid, TimeScale};
use deep_causality::types::context_types::node_types::space_tempoid::SpaceTempoid;
use crate::types::dateoid::Dataoid;
use crate::types::sampled_date_time_bar::SampledDataBars;

pub fn generate_time_data_graph(
    data: &SampledDataBars,
    time_scale: TimeScale,
)
    -> Result<ContextMatrixGraph<Dataoid, Spaceoid, Tempoid, SpaceTempoid>, Box<dyn Error>>
{

}

fn get_boolean_control_map(
    time_scale: TimeScale
)
    -> Vec<bool>
{
    return match time_scale {
        // Boolean Index:
        // 0: Year,1: Quarter,2: Month,3: Week,4: Day,5: Hour,6: Minute, 7: Second
        TimeScale::NoScale =>  vec![true, true, true, true, true, true, true, true],
        TimeScale::Second =>  vec![true, true, true, true, true, true, true, true],
        TimeScale::Minute => vec![true, true, true, true, true, true, true, false],
        TimeScale::Hour => vec![true, true, true, true, true, true, false, false],
        TimeScale::Day => vec![true, true, true, true, true, false, false, false],
        TimeScale::Week => vec![true, true, true, true, false, false, false, false],
        TimeScale::Month => vec![true, true, true, false, false, false, false, false],
        TimeScale::Quarter => vec![true, true, false, false, false, false, false, false],
        TimeScale::Year => vec![true, false, false, false, false, false, false, false],
    };
}