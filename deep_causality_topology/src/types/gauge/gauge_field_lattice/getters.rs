/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, Lattice, LatticeCell, LatticeGaugeField, LinkVariable};
use std::collections::HashMap;
use std::sync::Arc;

impl<G: GaugeGroup, const D: usize, M, R> LatticeGaugeField<G, D, M, R> {
    /// The underlying lattice (dereferenced for convenience).
    ///
    /// # Returns
    ///
    /// Reference to the inner `Lattice` struct.
    #[inline]
    pub fn lattice(&self) -> &Lattice<D> {
        &self.lattice
    }

    /// The underlying lattice as Arc (for cloning).
    ///
    /// # Returns
    ///
    /// Reference to the `Arc<Lattice>`.
    #[inline]
    pub fn lattice_arc(&self) -> &Arc<Lattice<D>> {
        &self.lattice
    }

    /// Coupling parameter β = 2N/g² (reference).
    ///
    /// # Returns
    ///
    /// Reference to beta.
    #[inline]
    pub fn beta(&self) -> &R {
        &self.beta
    }

    /// Consume self and return the beta value.
    ///
    /// Useful for HKT operations that need to transform beta.
    ///
    /// # Returns
    ///
    /// The beta value.
    #[inline]
    pub fn beta_owned(self) -> R {
        self.beta
    }

    /// Consume self and return all components.
    ///
    /// # Returns
    ///
    /// Tuple of (lattice, links, beta).
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn into_parts(
        self,
    ) -> (
        Arc<Lattice<D>>,
        HashMap<LatticeCell<D>, LinkVariable<G, M, R>>,
        R,
    ) {
        (self.lattice, self.links, self.beta)
    }

    /// Number of links (edges).
    ///
    /// # Returns
    ///
    /// Total number of stored links.
    #[inline]
    pub fn num_links(&self) -> usize {
        self.links.len()
    }

    /// Get link variable for an edge.
    ///
    /// # Arguments
    ///
    /// * `edge` - The edge cell key
    ///
    /// # Returns
    ///
    /// Option containing reference to the link variable if present.
    #[inline]
    pub fn link(&self, edge: &LatticeCell<D>) -> Option<&LinkVariable<G, M, R>> {
        self.links.get(edge)
    }

    /// Mutable access to a link (for Monte Carlo updates).
    ///
    /// # Arguments
    ///
    /// * `edge` - The edge cell key
    ///
    /// # Returns
    ///
    /// Option containing mutable reference to the link variable.
    #[inline]
    pub fn link_mut(&mut self, edge: &LatticeCell<D>) -> Option<&mut LinkVariable<G, M, R>> {
        self.links.get_mut(edge)
    }

    /// Get all links as a reference to the internal HashMap.
    ///
    /// # Returns
    ///
    /// Reference to the links map.
    #[inline]
    pub fn links(&self) -> &HashMap<LatticeCell<D>, LinkVariable<G, M, R>> {
        &self.links
    }

    /// Set a specific link variable.
    ///
    /// # Arguments
    ///
    /// * `edge` - The edge cell key
    /// * `link` - The new link variable
    #[inline]
    pub fn set_link(&mut self, edge: LatticeCell<D>, link: LinkVariable<G, M, R>) {
        self.links.insert(edge, link);
    }
}
