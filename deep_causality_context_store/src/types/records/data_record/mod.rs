/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::SubstrateRef;

/// The payload of a data node, as a store holds it.
///
/// A closed value tree: a payload of any shape reduces to it, so a store holds the payload
/// without knowing its type. `Fields` is an ordered list rather than a map, so two records with
/// the same entries in the same order are equal and two in a different order are not. Which
/// variants a backend accepts is the backend's policy.
#[derive(Debug, Clone, PartialEq)]
pub enum DataRecord {
    Number(f64),
    Count(u64),
    Integer(i64),
    Flag(bool),
    Text(String),
    /// Where the value lives when the store holds structure and not values.
    Reference(SubstrateRef),
    List(Vec<DataRecord>),
    Fields(Vec<(String, DataRecord)>),
}

impl DataRecord {
    /// The variant's name, for an error that says what was found.
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Number(_) => "Number",
            Self::Count(_) => "Count",
            Self::Integer(_) => "Integer",
            Self::Flag(_) => "Flag",
            Self::Text(_) => "Text",
            Self::Reference(_) => "Reference",
            Self::List(_) => "List",
            Self::Fields(_) => "Fields",
        }
    }
}
