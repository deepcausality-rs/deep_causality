/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextId, ContextoidId};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// A refusal of the in-memory backend, one variant per rule of the storage contract it enforces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryStorageError(pub MemoryStorageErrorEnum);

impl Error for MemoryStorageError {}

/// The classification of an in-memory backend refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryStorageErrorEnum {
    /// The store holds no container under this identifier.
    UnknownContext(ContextId),
    /// The store holds no contextoid under this identifier.
    UnknownNode(ContextoidId),
    /// The store holds no relation between these two contextoids.
    UnknownEdge {
        from: ContextoidId,
        to: ContextoidId,
    },
    /// `create_node` was given an identifier the store never handed out through `reserve`.
    IdentityNotReserved(ContextoidId),
    /// The store holds this identifier under a different record.
    NodeConflict(ContextoidId),
    /// The store holds a relation of a different kind between these two contextoids.
    EdgeConflict {
        from: ContextoidId,
        to: ContextoidId,
    },
    /// A container cannot reference itself.
    SelfReference(ContextId),
    /// The event names no operation a host may perform: a container's identifier is the store's,
    /// and a view's answer moving has no operation behind it.
    EventNotApplicable(&'static str),
    /// A cursor past the end of the log.
    UnknownCursor(usize),
}

impl MemoryStorageError {
    pub const fn new(kind: MemoryStorageErrorEnum) -> Self {
        Self(kind)
    }

    pub const fn kind(&self) -> &MemoryStorageErrorEnum {
        &self.0
    }

    #[allow(non_snake_case)]
    pub const fn UnknownContext(context: ContextId) -> Self {
        Self(MemoryStorageErrorEnum::UnknownContext(context))
    }

    #[allow(non_snake_case)]
    pub const fn UnknownNode(node: ContextoidId) -> Self {
        Self(MemoryStorageErrorEnum::UnknownNode(node))
    }

    #[allow(non_snake_case)]
    pub const fn UnknownEdge(from: ContextoidId, to: ContextoidId) -> Self {
        Self(MemoryStorageErrorEnum::UnknownEdge { from, to })
    }

    #[allow(non_snake_case)]
    pub const fn IdentityNotReserved(node: ContextoidId) -> Self {
        Self(MemoryStorageErrorEnum::IdentityNotReserved(node))
    }

    #[allow(non_snake_case)]
    pub const fn NodeConflict(node: ContextoidId) -> Self {
        Self(MemoryStorageErrorEnum::NodeConflict(node))
    }

    #[allow(non_snake_case)]
    pub const fn EdgeConflict(from: ContextoidId, to: ContextoidId) -> Self {
        Self(MemoryStorageErrorEnum::EdgeConflict { from, to })
    }

    #[allow(non_snake_case)]
    pub const fn SelfReference(context: ContextId) -> Self {
        Self(MemoryStorageErrorEnum::SelfReference(context))
    }

    #[allow(non_snake_case)]
    pub const fn EventNotApplicable(event: &'static str) -> Self {
        Self(MemoryStorageErrorEnum::EventNotApplicable(event))
    }

    #[allow(non_snake_case)]
    pub const fn UnknownCursor(cursor: usize) -> Self {
        Self(MemoryStorageErrorEnum::UnknownCursor(cursor))
    }
}

impl Display for MemoryStorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            MemoryStorageErrorEnum::UnknownContext(context) => {
                write!(f, "MemoryStorageError: no container {context}")
            }
            MemoryStorageErrorEnum::UnknownNode(node) => {
                write!(f, "MemoryStorageError: no contextoid {node}")
            }
            MemoryStorageErrorEnum::UnknownEdge { from, to } => {
                write!(f, "MemoryStorageError: no relation from {from} to {to}")
            }
            MemoryStorageErrorEnum::IdentityNotReserved(node) => write!(
                f,
                "MemoryStorageError: identifier {node} was not handed out by reserve"
            ),
            MemoryStorageErrorEnum::NodeConflict(node) => write!(
                f,
                "MemoryStorageError: contextoid {node} is held under a different record"
            ),
            MemoryStorageErrorEnum::EdgeConflict { from, to } => write!(
                f,
                "MemoryStorageError: a relation of another kind exists from {from} to {to}"
            ),
            MemoryStorageErrorEnum::SelfReference(context) => {
                write!(
                    f,
                    "MemoryStorageError: container {context} cannot reference itself"
                )
            }
            MemoryStorageErrorEnum::EventNotApplicable(event) => {
                write!(
                    f,
                    "MemoryStorageError: {event} names no operation a host may apply"
                )
            }
            MemoryStorageErrorEnum::UnknownCursor(cursor) => {
                write!(
                    f,
                    "MemoryStorageError: cursor {cursor} is past the end of the log"
                )
            }
        }
    }
}
