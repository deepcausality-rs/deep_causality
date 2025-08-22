/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Datable, SpaceTemporal, Spatial, Symbolic, Teloid, TeloidID, Temporal};
use std::collections::HashMap;

mod store;

/// A generic, in-memory storage for Teloids, indexed by their unique ID.
#[derive(Debug, Default, Clone)]
#[allow(clippy::type_complexity)]
pub struct TeloidStore<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    index: HashMap<TeloidID, Teloid<D, S, T, ST, SYM, VS, VT>>,
}

impl<D, S, T, ST, SYM, VS, VT> TeloidStore<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
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
