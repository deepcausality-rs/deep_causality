/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, ProjectionError};

/// A context node type that one record describes completely, in both directions.
///
/// The record is a type parameter rather than an associated type so that one node type can
/// project onto more than one record; the absent spacetime fills both the space and the spacetime
/// slot of a context and implements this for each. `to_record` is fallible for the same reason:
/// a node type with no record answers `ProjectionError::Unrecordable` rather than inventing one.
///
/// The trait is declared here and implemented beside each node type in the context crate, so a
/// change to a node type's shape fails that crate's build rather than a backend's data.
pub trait Recordable<Rec>: Sized {
    /// The node as a record.
    fn to_record(&self) -> Result<Rec, ProjectionError>;

    /// The node a record describes, under the given identifier.
    fn from_record(id: ContextoidId, record: Rec) -> Result<Self, ProjectionError>;
}
