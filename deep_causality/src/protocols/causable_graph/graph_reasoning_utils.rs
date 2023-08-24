// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::collections::HashMap;

use crate::prelude::{IdentificationValue, NumericalValue};

pub(crate) fn get_obs<'a>(
    cause_id: IdentificationValue,
    data: &'a [NumericalValue],
    data_index: &'a Option<&HashMap<IdentificationValue, IdentificationValue>>,
) -> NumericalValue {
    let obs = if data_index.is_some() {
        data.get(
            *data_index
                .unwrap()
                .get(&cause_id)
                .expect("Failed to get data index") as usize,
        )
        .expect("Failed to get data")
    } else {
        data.get(cause_id as usize).expect("Failed to get data")
    };

    obs.to_owned()
}
