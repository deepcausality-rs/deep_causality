/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, LatticeGaugeField, LinkVariable};
use std::collections::HashMap;
// ============================================================================
// Gauge Transformations
// ============================================================================
impl<G: GaugeGroup, const D: usize, T: Clone + Default> LatticeGaugeField<G, D, T> {
    /// Apply a gauge transformation: U_μ(n) → g(n) U_μ(n) g(n+μ)†.
    ///
    /// # Arguments
    /// * `gauge_fn` - Function providing g(n) for each lattice site
    pub fn gauge_transform<F>(&mut self, gauge_fn: F)
    where
        F: Fn(&[usize; D]) -> LinkVariable<G, T>,
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Neg<Output = T>,
    {
        let shape = self.lattice.shape();
        let new_links: HashMap<_, _> = self
            .links
            .iter()
            .map(|(cell, u)| {
                let site = *cell.position();

                // Find direction of this edge
                let dir = cell.orientation().trailing_zeros() as usize;

                // Get g(n)
                let g_n = gauge_fn(&site);

                // Get n + μ̂
                let mut site_plus_mu = site;
                site_plus_mu[dir] = (site_plus_mu[dir] + 1) % shape[dir];

                // Get g(n+μ)†
                let g_n_plus_mu_dag = gauge_fn(&site_plus_mu).dagger();

                // U' = g(n) U g(n+μ)†
                let new_u = g_n.mul(u).mul(&g_n_plus_mu_dag);

                (cell.clone(), new_u)
            })
            .collect();

        self.links = new_links;
    }
}
