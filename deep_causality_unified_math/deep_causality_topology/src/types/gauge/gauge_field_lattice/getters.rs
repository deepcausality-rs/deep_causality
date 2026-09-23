/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::utils::{link_cell, link_index};
use crate::{GaugeGroup, LatticeCell, LatticeComplex, LatticeGaugeField, LinkVariable};
use std::sync::Arc;

impl<G: GaugeGroup, const D: usize, M, R: deep_causality_algebra::RealField, S>
    LatticeGaugeField<G, D, M, R, S>
{
    /// The underlying lattice (dereferenced for convenience).
    ///
    /// # Returns
    ///
    /// Reference to the inner `LatticeComplex` struct.
    #[inline]
    pub fn lattice(&self) -> &LatticeComplex<D, R> {
        &self.lattice
    }

    /// The underlying lattice as Arc (for cloning).
    ///
    /// # Returns
    ///
    /// Reference to the `Arc<LatticeComplex>`.
    #[inline]
    pub fn lattice_arc(&self) -> &Arc<LatticeComplex<D, R>> {
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
    /// Tuple of (lattice, links, beta). `links` has one slot per site per direction, indexed
    /// `site_offset * D + mu` with `site_offset` row-major over the lattice shape.
    #[inline]
    #[allow(clippy::type_complexity)]
    pub fn into_parts(
        self,
    ) -> (
        Arc<LatticeComplex<D, R>>,
        Vec<Option<LinkVariable<G, M, R>>>,
        R,
    ) {
        (self.lattice, self.links, self.beta)
    }

    /// Number of links (edges).
    ///
    /// # Returns
    ///
    /// Total number of stored links.
    ///
    /// Counts occupied slots: the table has one slot per site per direction, and a field built
    /// from a partial edge map leaves the rest empty.
    #[inline]
    pub fn num_links(&self) -> usize {
        self.links.iter().filter(|s| s.is_some()).count()
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
        let i = link_index(&self.lattice, edge)?;
        self.links.get(i)?.as_ref()
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
        let i = link_index(&self.lattice, edge)?;
        self.links.get_mut(i)?.as_mut()
    }

    /// Every link the field carries, paired with the edge it sits on.
    ///
    /// Sites come in row-major order over the lattice shape (last axis fastest), and at each site
    /// the directions in ascending axis order. This is not the order of `lattice.cells(1)`,
    /// which is direction-major with axis 0 fastest.
    #[inline]
    pub fn iter_links(&self) -> impl Iterator<Item = (LatticeCell<D>, &LinkVariable<G, M, R>)> {
        let shape = *self.lattice.shape();
        self.links
            .iter()
            .enumerate()
            .filter_map(move |(i, slot)| slot.as_ref().map(|l| (link_cell(&shape, i), l)))
    }

    /// The edges carrying a link, in the order of [`iter_links`](Self::iter_links).
    #[inline]
    pub fn link_cells(&self) -> Vec<LatticeCell<D>> {
        self.iter_links().map(|(cell, _)| cell).collect()
    }

    /// Whether the field carries no links at all.
    #[inline]
    pub fn has_no_links(&self) -> bool {
        self.links.iter().all(|s| s.is_none())
    }

    /// Set a specific link variable.
    ///
    /// Does nothing if `edge` is not an edge of the lattice, including an edge that would leave
    /// the lattice across a non-periodic boundary.
    ///
    /// # Arguments
    ///
    /// * `edge` - The edge cell key
    /// * `link` - The new link variable
    #[inline]
    pub fn set_link(&mut self, edge: LatticeCell<D>, link: LinkVariable<G, M, R>) {
        if let Some(i) = link_index(&self.lattice, &edge) {
            self.links[i] = Some(link);
        }
    }
}
