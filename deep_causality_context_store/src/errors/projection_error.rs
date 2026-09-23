/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, IdentificationValue};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Why a record could not be read into a node type, or a node type written into a record.
///
/// A public struct around a public enum, so the classification can grow without the struct
/// changing. Every variant that concerns a node names it, so the failure points at one contextoid
/// rather than at a snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionError(pub ProjectionErrorEnum);

impl Error for ProjectionError {}

/// The classification of a projection failure.
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionErrorEnum {
    /// The record's variant is one this node type cannot hold: a `Geo` record read into a
    /// `EuclideanSpace`.
    WrongVariant {
        id: ContextoidId,
        expected: &'static str,
        found: &'static str,
    },
    /// The data record's variant is one this payload cannot hold: a `Count` read into an `f64`.
    WrongPayload {
        id: ContextoidId,
        expected: &'static str,
        found: &'static str,
    },
    /// A `Fields` record lacks an entry the payload expects.
    MissingField {
        id: ContextoidId,
        field: &'static str,
    },
    /// The node has no record: an absent spacetime, or the phantom arm of a contextoid.
    Unrecordable {
        id: ContextoidId,
        kind: &'static str,
    },
    /// The record's scalar has no value in the node's scalar type.
    Scalar { id: ContextoidId, value: f64 },
    /// An identifier breaks a rule of the snapshot: named by no node, carried twice, or 0 for an
    /// extra context. `rule` states which.
    Identity {
        id: IdentificationValue,
        rule: &'static str,
    },
    /// The snapshot was written under a newer record version than this reader supports.
    Version { found: u16, supported: u16 },
}

impl ProjectionError {
    pub const fn new(kind: ProjectionErrorEnum) -> Self {
        Self(kind)
    }

    pub const fn kind(&self) -> &ProjectionErrorEnum {
        &self.0
    }

    #[allow(non_snake_case)]
    pub const fn WrongVariant(
        id: ContextoidId,
        expected: &'static str,
        found: &'static str,
    ) -> Self {
        Self(ProjectionErrorEnum::WrongVariant {
            id,
            expected,
            found,
        })
    }

    #[allow(non_snake_case)]
    pub const fn WrongPayload(
        id: ContextoidId,
        expected: &'static str,
        found: &'static str,
    ) -> Self {
        Self(ProjectionErrorEnum::WrongPayload {
            id,
            expected,
            found,
        })
    }

    #[allow(non_snake_case)]
    pub const fn MissingField(id: ContextoidId, field: &'static str) -> Self {
        Self(ProjectionErrorEnum::MissingField { id, field })
    }

    #[allow(non_snake_case)]
    pub const fn Unrecordable(id: ContextoidId, kind: &'static str) -> Self {
        Self(ProjectionErrorEnum::Unrecordable { id, kind })
    }

    #[allow(non_snake_case)]
    pub const fn Scalar(id: ContextoidId, value: f64) -> Self {
        Self(ProjectionErrorEnum::Scalar { id, value })
    }

    #[allow(non_snake_case)]
    pub const fn Identity(id: IdentificationValue, rule: &'static str) -> Self {
        Self(ProjectionErrorEnum::Identity { id, rule })
    }

    #[allow(non_snake_case)]
    pub const fn Version(found: u16, supported: u16) -> Self {
        Self(ProjectionErrorEnum::Version { found, supported })
    }
}

impl Display for ProjectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            ProjectionErrorEnum::WrongVariant {
                id,
                expected,
                found,
            } => write!(
                f,
                "ProjectionError: node {id} expected variant {expected}, found {found}"
            ),
            ProjectionErrorEnum::WrongPayload {
                id,
                expected,
                found,
            } => write!(
                f,
                "ProjectionError: node {id} expected payload {expected}, found {found}"
            ),
            ProjectionErrorEnum::MissingField { id, field } => {
                write!(f, "ProjectionError: node {id} lacks field {field}")
            }
            ProjectionErrorEnum::Unrecordable { id, kind } => {
                write!(f, "ProjectionError: node {id} of kind {kind} has no record")
            }
            ProjectionErrorEnum::Scalar { id, value } => write!(
                f,
                "ProjectionError: node {id} cannot hold the scalar {value}"
            ),
            ProjectionErrorEnum::Identity { id, rule } => {
                write!(
                    f,
                    "ProjectionError: identifier {id} breaks the rule: {rule}"
                )
            }
            ProjectionErrorEnum::Version { found, supported } => write!(
                f,
                "ProjectionError: record version {found} is newer than the supported {supported}"
            ),
        }
    }
}
