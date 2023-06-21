/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */


use crate::prelude::{Adjustable, Identifiable};

pub trait Spatial: Identifiable + Adjustable {
    fn x(&self) -> i64;
    fn y(&self) -> i64;
    fn z(&self) -> i64;
}
