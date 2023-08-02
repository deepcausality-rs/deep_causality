// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::model::causaloid_year::get_year_causaloid;
use deep_causality::prelude::{Causaloid, Context, Spaceoid, SpaceTempoid, Tempoid};
use crate::types::dateoid::Dataoid;

pub mod utils;
pub mod causaloid_year;

pub fn get_causaloid<'l>(
    context: &'l Context<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>
)
    -> Causaloid<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>
{
    get_year_causaloid(context)
}