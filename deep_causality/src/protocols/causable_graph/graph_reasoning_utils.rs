// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::collections::HashMap;

use crate::errors::CausalityGraphError;
use crate::prelude::{Causable, IdentificationValue, NumericalValue};

pub(crate) fn get_obs<'a>(
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

pub(crate) fn verify_cause(
    causaloid: &impl Causable,
    data: &[NumericalValue],
)
    -> Result<bool, CausalityGraphError>
{
    if data.is_empty() {
        return Err(CausalityGraphError("Data are empty (len=0)".into()));
    }

    if data.len() == 1 {
        let obs = data.first().expect("Failed to get data");
        return match causaloid.verify_single_cause(obs) {
            Ok(res) => Ok(res),
            Err(e) => Err(CausalityGraphError(e.0)),
        };
    }

    if data.len() > 1 {
        for obs in data.iter() {
            if !causaloid.verify_single_cause(obs).expect("Failed to verify data") {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
