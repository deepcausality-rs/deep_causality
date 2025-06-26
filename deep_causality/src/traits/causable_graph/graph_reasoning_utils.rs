/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashMap;

use crate::prelude::{IdentificationValue, NumericalValue};

/// Gets the observation value for a cause from the given data.
///
/// cause_id: The ID of the cause to get observations for
/// data: Array of observation values
/// data_index: Optional map from node IDs to indices into data
///
/// If data_index is provided, uses it to lookup index for cause_id.
/// Else assumes cause_id maps directly to index in data.
///
/// Returns the observation value for the cause.
///
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
