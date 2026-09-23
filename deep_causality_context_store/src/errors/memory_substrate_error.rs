/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, SubstrateRef};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// A refusal of the in-memory substrate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySubstrateError(pub MemorySubstrateErrorEnum);

impl Error for MemorySubstrateError {}

/// The classification of an in-memory substrate refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemorySubstrateErrorEnum {
    /// `deposit` was given a `Reference`; a reference to a reference names nothing.
    ReferenceRefused(ContextoidId),
    /// `resolve` was given a reference the substrate does not hold.
    UnknownReference(SubstrateRef),
}

impl MemorySubstrateError {
    pub const fn new(kind: MemorySubstrateErrorEnum) -> Self {
        Self(kind)
    }

    pub const fn kind(&self) -> &MemorySubstrateErrorEnum {
        &self.0
    }

    #[allow(non_snake_case)]
    pub const fn ReferenceRefused(node: ContextoidId) -> Self {
        Self(MemorySubstrateErrorEnum::ReferenceRefused(node))
    }

    #[allow(non_snake_case)]
    pub const fn UnknownReference(reference: SubstrateRef) -> Self {
        Self(MemorySubstrateErrorEnum::UnknownReference(reference))
    }
}

impl Display for MemorySubstrateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            MemorySubstrateErrorEnum::ReferenceRefused(node) => write!(
                f,
                "MemorySubstrateError: node {node} deposited a reference, which names no value"
            ),
            MemorySubstrateErrorEnum::UnknownReference(reference) => {
                write!(f, "MemorySubstrateError: no value under {reference}")
            }
        }
    }
}
