// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use std::error::Error;
use crate::types::sampled_date_time_bar::SampledDataBars;

pub fn load_agg_data()
    -> Result<SampledDataBars, Box<dyn Error>>
{
    Ok(SampledDataBars::default())
}