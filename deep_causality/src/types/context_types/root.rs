// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use std::fmt::{Display, Formatter};

use crate::protocols::identifiable::Identifiable;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Root {
    id: u64,
}

impl Root
{
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

impl Identifiable for Root
{
    fn id(&self) -> u64 {
        self.id
    }
}

impl Display for Root
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Root ID: {}",
               self.id,
        )
    }
}
