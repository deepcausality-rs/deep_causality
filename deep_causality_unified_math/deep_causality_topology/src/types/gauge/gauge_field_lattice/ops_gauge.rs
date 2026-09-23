/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
//! Gauge field operations.
//!
//! The gauge transformation of a field by a site-wise group element. A random transformation
//! built on it lives in `ops_gauge_transform.rs`.

use crate::{GaugeGroup, LatticeGaugeField, LinkVariable};
use deep_causality_algebra::{ComplexField, DivisionAlgebra, Field, RealField};
use deep_causality_num::{FromPrimitive, ToPrimitive};

use super::utils::{alloc_slots, link_index};
use std::fmt::Debug;

// ============================================================================
// Gauge Transformations
// ============================================================================
impl<
    G: GaugeGroup,
    const D: usize,
    M: Field + Copy + Default + PartialOrd + Debug + ComplexField<R> + DivisionAlgebra<R>,
    R: RealField + FromPrimitive + ToPrimitive,
    S,
> LatticeGaugeField<G, D, M, R, S>
{
    /// Apply a gauge transformation.
    ///
    /// # Mathematics
    ///
    /// $$U_\mu(x) \to \Omega(x) U_\mu(x) \Omega^\dagger(x+\hat\mu)$$
    ///
    /// # Physics
    ///
    /// Local basis rotation in the internal symmetry space.
    ///
    /// # Arguments
    ///
    /// * `gauge_fn` - Closure providing $\Omega(x)$ for each site. It is called exactly once per
    ///   site, in row-major site order (last axis fastest), so every link touching a site sees
    ///   the same $\Omega(x)$ even when the closure returns a different element on each call.
    ///
    /// # Returns
    ///
    /// None (modifies field in-place).
    pub fn gauge_transform<F>(&mut self, gauge_fn: F)
    where
        F: Fn(&[usize; D]) -> LinkVariable<G, M, R>,
        M: Field + DivisionAlgebra<R>,
        R: RealField,
    {
        let shape = *self.lattice.shape();

        // Ω(x) for every site, indexed row-major over the shape.
        let num_sites: usize = shape.iter().product();
        let site_of = |mut offset: usize| {
            let mut site = [0usize; D];
            for d in (0..D).rev() {
                site[d] = offset % shape[d];
                offset /= shape[d];
            }
            site
        };
        let offset_of = |site: &[usize; D]| {
            site.iter()
                .zip(shape.iter())
                .fold(0, |acc, (p, l)| acc * l + p)
        };
        let omega: Vec<LinkVariable<G, M, R>> =
            (0..num_sites).map(|o| gauge_fn(&site_of(o))).collect();

        let new_links: Vec<Option<LinkVariable<G, M, R>>> = self
            .iter_links()
            .map(|(cell, u)| {
                let site = *cell.position();
                let dir = cell.orientation().trailing_zeros() as usize;

                // n + μ̂, wrapped periodically.
                let mut site_plus_mu = site;
                site_plus_mu[dir] = (site_plus_mu[dir] + 1) % shape[dir];

                // U' = Ω(n) U Ω(n+μ̂)†
                let new_u = omega[offset_of(&site)]
                    .mul(u)
                    .mul(&omega[offset_of(&site_plus_mu)].dagger());

                (cell, new_u)
            })
            .fold(alloc_slots(&shape), |mut slots, (cell, u)| {
                if let Some(i) = link_index(&self.lattice, &cell) {
                    slots[i] = Some(u);
                }
                slots
            });

        self.links = new_links;
    }
}
