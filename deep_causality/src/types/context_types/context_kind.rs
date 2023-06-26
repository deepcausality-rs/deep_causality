// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ContextKind {
    Time,
    Space,
    SpaceTime,
    Data,
    TimeData,
    SpaceData,
    SpaceTimeData,
}

impl Display for ContextKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextKind::Time => write!(f, "Time"),
            ContextKind::Space => write!(f, "Space"),
            ContextKind::SpaceTime => write!(f, "SpaceTime"),
            ContextKind::Data => write!(f, "Data"),
            ContextKind::TimeData => write!(f, "TimeData"),
            ContextKind::SpaceData => write!(f, "SpaceData"),
            ContextKind::SpaceTimeData => write!(f, "SpaceTimeData"),
        }
    }
}
