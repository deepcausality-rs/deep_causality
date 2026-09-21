/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Teloid, TeloidID};
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};
use std::collections::HashMap;

mod store;

/// A generic, in-memory storage for Teloids, indexed by their unique ID.
#[derive(Debug, Default, Clone)]
#[allow(clippy::type_complexity)]
pub struct TeloidStore<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    index: HashMap<TeloidID, Teloid<D, S, T, ST>>,
}

impl<D, S, T, ST> TeloidStore<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    /// Creates a new, empty `TeloidStore`.
    ///
    /// # Returns
    ///
    /// A new `TeloidStore` instance.
    ///
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Creates a new `TeloidStore` with a specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The initial capacity of the store.
    ///
    /// # Returns
    ///
    /// A new `TeloidStore` instance with the given capacity.
    ///
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
        }
    }
}
