// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use std::error::Error;
use deep_causality::prelude::{Context, Model, Spaceoid, SpaceTempoid, Tempoid};
use crate::model::causaloid_year::get_year_causaloid;
use crate::types::dateoid::Dataoid;

pub fn build_model<'l>(
    context: &'l Context<Dataoid, Spaceoid, Tempoid, SpaceTempoid>
)
    -> Result<Model<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>, Box<dyn Error>>
{

    let id = 1;
    let author = "Marvin Hansen <marvin.hansen@gmail.com>";
    let description = "This is a test model";
    let assumptions = None;
    let causaloid = get_year_causaloid(context);

    Ok(
        Model::new(
            id,
            author,
            description,
            assumptions,
            &causaloid,
            Some(context)
        )
    )
}