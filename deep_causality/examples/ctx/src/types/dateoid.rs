// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Display, Formatter};
use deep_causality::prelude::{Adjustable, Datable, Identifiable};
use crate::protocols::rangeable::Rangeable;
use crate::types::bar_range::BarRange;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Dataoid
{
    id: u64,
    data_range: BarRange,
}

impl Dataoid
{
    pub fn new(id: u64, data_range: BarRange) -> Self {
        Self { id, data_range }
    }
}
impl Adjustable for Dataoid {}

impl Datable for Dataoid {}

impl Identifiable for Dataoid
{
    fn id(&self) -> u64 {
        self.id
    }
}

impl Rangeable for Dataoid
{
    fn data_range(&self) -> BarRange {
        self.data_range
    }
}

impl Display for Dataoid
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {} range: {}", self.id, self.data_range)
    }
}