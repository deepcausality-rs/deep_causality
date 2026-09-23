/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::convert::Infallible;
use deep_causality_context_store::ProjectionError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// Why a store operation failed: the backend refused, the substrate refused, or the projection
/// between records and nodes did.
///
/// `B` defaults to `Infallible`, so a path with no substrate reads `StoreError<S::Error>` and
/// the substrate paths read `StoreError<S::Error, B::Error>` with one type.
#[derive(Debug, Clone, PartialEq)]
pub struct StoreError<E, B = Infallible>(pub StoreErrorEnum<E, B>);

impl<E: Debug + Display, B: Debug + Display> Error for StoreError<E, B> {}

/// The classification of a store failure.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreErrorEnum<E, B> {
    /// The backend refused.
    Storage(E),
    /// The substrate refused.
    Substrate(B),
    /// A record could not be read into a node type, or a node written into a record.
    Projection(ProjectionError),
}

impl<E, B> StoreError<E, B> {
    pub const fn new(kind: StoreErrorEnum<E, B>) -> Self {
        Self(kind)
    }

    pub const fn kind(&self) -> &StoreErrorEnum<E, B> {
        &self.0
    }

    #[allow(non_snake_case)]
    pub const fn Storage(error: E) -> Self {
        Self(StoreErrorEnum::Storage(error))
    }

    #[allow(non_snake_case)]
    pub const fn Substrate(error: B) -> Self {
        Self(StoreErrorEnum::Substrate(error))
    }

    #[allow(non_snake_case)]
    pub const fn Projection(error: ProjectionError) -> Self {
        Self(StoreErrorEnum::Projection(error))
    }
}

impl<E, B> From<ProjectionError> for StoreError<E, B> {
    fn from(error: ProjectionError) -> Self {
        Self::Projection(error)
    }
}

impl<E: Display, B: Display> Display for StoreError<E, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            StoreErrorEnum::Storage(error) => write!(f, "StoreError: storage: {error}"),
            StoreErrorEnum::Substrate(error) => write!(f, "StoreError: substrate: {error}"),
            StoreErrorEnum::Projection(error) => write!(f, "StoreError: projection: {error}"),
        }
    }
}
