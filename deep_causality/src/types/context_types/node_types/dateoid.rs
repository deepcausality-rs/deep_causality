// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.


use std::fmt::{Display, Formatter};

use crate::prelude::{Datable, Identifiable};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Dataoid
{
    id: u64,
    data: i32,
}


impl Dataoid
{
    pub fn new(id: u64, data_range: i32) -> Self {
        Self { id, data: data_range }
    }
    pub fn data(&self) -> i32 {
        self.data
    }
}

// Optional. Override only when needed.
// impl Adjustable for Dataoid {}

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
        write!(f, "Dataoid: id: {} data: {}", self.id, self.data)
    }
}