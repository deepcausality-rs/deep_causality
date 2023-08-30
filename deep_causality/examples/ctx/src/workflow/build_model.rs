// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::error::Error;

use deep_causality::prelude::{Causaloid, Context, Model, SpaceTempoid, Spaceoid, Tempoid};

use crate::types::dateoid::Dataoid;

pub fn build_model<'l>(
    context: &'l Context<Dataoid, Spaceoid, Tempoid, SpaceTempoid>,
    causaloid: &'l Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>,
) -> Result<Model<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>, Box<dyn Error>> {
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
