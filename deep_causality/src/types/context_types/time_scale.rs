// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum TimeScale
{
    #[default]
    NoScale,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl Display for TimeScale
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}
