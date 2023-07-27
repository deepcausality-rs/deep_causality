// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use std::error::Error;
use deep_causality::prelude::{BuildError, CausalFn, Causaloid, DescriptionValue, IdentificationValue};

// Builds a new singleton Causaloid.
pub fn build_causaloid(
    id: IdentificationValue,
    causal_fn: CausalFn,
    description: DescriptionValue,
    data_set_id: DescriptionValue,
)
    -> Result<Causaloid, Box<dyn Error>>
{
    // check description
    if description.is_empty() {
        return Err(Box::new(BuildError("Description empty".into())));
    }

    // check data_set_id
    if data_set_id.is_empty() {
        return Err(Box::new(BuildError("Data set identifier empty".into())));
    }

    Ok(
        Causaloid::new(
            id,
            causal_fn,
            data_set_id,
            description,
        )
    )
}
