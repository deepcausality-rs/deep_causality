/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
//! Gauge field operations.
//!
//! Includes legacy gauge transformations and staple calculations.
//! Note: For robust gauge transformations with error handling, see `ops_gauge_transform.rs`.

use crate::{GaugeGroup, LatticeGaugeField, LinkVariable};
use deep_causality_tensor::TensorData;
use std::collections::HashMap;

// ============================================================================
// Gauge Transformations
// ============================================================================
impl<G: GaugeGroup, const D: usize, T: TensorData> LatticeGaugeField<G, D, T> {
    /// Apply a gauge transformation (infallible version).
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
    /// * `gauge_fn` - Closure providing $\Omega(x)$ for each site
    ///
    /// # Returns
    ///
    /// None (modifies field in-place).
    pub fn gauge_transform<F>(&mut self, gauge_fn: F)
    where
        F: Fn(&[usize; D]) -> LinkVariable<G, T>,
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
