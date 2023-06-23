/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
use std::fmt::{Debug, Display};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum RelationKind {
    Datial,
    Temporal,
    Spatial,
    SpaceTemporal,
}

impl Display for RelationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationKind::Datial => write!(f, "Datial"),
            RelationKind::Temporal => write!(f, "Temporal"),
            RelationKind::Spatial => write!(f, "Spatial"),
            RelationKind::SpaceTemporal => write!(f, "SpaceTemporal"),
        }
    }
}
