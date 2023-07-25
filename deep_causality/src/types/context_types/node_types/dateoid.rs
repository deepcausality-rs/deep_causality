// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use std::fmt::{Display, Formatter};
use crate::prelude::{Adjustable, Datable, Identifiable};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Dataoid
{
    id: u64,
    data_range: i32,
}


impl Dataoid
{
    pub fn new(id: u64, data_range: i32) -> Self {
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


impl Display for Dataoid
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {} range: {}", self.id, self.data_range)
    }
}