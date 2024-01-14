// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::error::Error;

use deep_causality::prelude::Model;

use crate::types::alias::{CustomCausaloid, CustomContext, CustomModel};

pub fn build_model<'l>(
    context: &'l CustomContext<'l>,
    causaloid: &'l CustomCausaloid<'l>,
) -> Result<CustomModel<'l>, Box<dyn Error>> {
    let id = 1;
    let author = "Marvin Hansen <marvin.hansen@gmail.com>";
    let description = "This is a test model";
    let assumptions = None;

    Ok(Model::new(
        id,
        author,
        description,
        assumptions,
        causaloid,
        Some(context),
    ))
}
