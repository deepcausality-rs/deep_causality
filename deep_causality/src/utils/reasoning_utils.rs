// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::collections::HashMap;
use crate::prelude::{IdentificationValue, NumericalValue};

pub fn append_string<'l>(
    s1: &'l mut String,
    s2: &'l str,
)
    -> &'l str
{
    s1.push('\n');
    s1.push_str(format!(" * {}", s2).as_str());
    s1.push('\n');

    s1
}

pub fn get_obs<'a>(
    cause_id: IdentificationValue,
    data: &'a [NumericalValue],
    data_index: &'a Option<&HashMap<IdentificationValue, IdentificationValue>>,
)
    -> NumericalValue
{
    let obs = if data_index.is_some()
    {
        let idx = data_index.unwrap().get(&cause_id).expect("Failed to get data index");
        let index = idx.to_owned() as usize;
        data.get(index).expect("Failed to get data")
    } else {
        let index = cause_id as usize;
        data.get(index).expect("Failed to get data")
    };

    obs.to_owned()
}
