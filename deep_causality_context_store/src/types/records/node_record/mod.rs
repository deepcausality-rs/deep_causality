/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DataRecord, SpaceRecord, SpaceTimeRecord, TimeRecord};

/// A contextoid's payload as a store holds it: one variant per `ContextoidType` variant.
///
/// A root is an ordinary record, stored and hydrated like any other node.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeRecord {
    Root,
    Data(DataRecord),
    Time(TimeRecord),
    Space(SpaceRecord),
    SpaceTime(SpaceTimeRecord),
}

impl NodeRecord {
    /// The variant's name, for an error that says what was found.
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Root => "Root",
            Self::Data(_) => "Data",
            Self::Time(_) => "Time",
            Self::Space(_) => "Space",
            Self::SpaceTime(_) => "SpaceTime",
        }
    }
}
