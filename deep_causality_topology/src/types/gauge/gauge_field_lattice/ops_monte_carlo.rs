/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GaugeGroup, LatticeCell, LatticeGaugeField, LinkVariable, TopologyError};

// ============================================================================
// Monte Carlo Updates
// ============================================================================
impl<G: GaugeGroup, const D: usize, T: Clone + Default> LatticeGaugeField<G, D, T> {
    /// Compute the staple sum for a given link.
    ///
    /// The staple V consists of all "horseshoe" paths around the link
    /// that complete plaquettes. For each plane containing the link direction,
    /// there are two staples (forward and backward).
    ///
    /// V = Σ_{ν≠μ} [ U_ν(n+μ̂) U_μ†(n+ν̂) U_ν†(n) + U_ν†(n+μ̂-ν̂) U_μ†(n-ν̂) U_ν(n-ν̂) ]
    ///
    /// # Errors
    ///
    /// Returns error if staple computation fails.
    pub fn try_staple(&self, edge: &LatticeCell<D>) -> Result<LinkVariable<G, T>, TopologyError>
    where
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Neg<Output = T>
            + From<f64>,
    {
        let site = *edge.position();
        let mu = edge.orientation().trailing_zeros() as usize;
        let shape = self.lattice.shape();

        let mut staple_sum = LinkVariable::<G, T>::try_zero().map_err(TopologyError::from)?;

        for nu in 0..D {
            if nu == mu {
                continue;
            }

            // Forward staple: U_ν(n+μ̂) U_μ†(n+ν̂) U_ν†(n)
            let mut site_plus_mu = site;
            site_plus_mu[mu] = (site_plus_mu[mu] + 1) % shape[mu];

            let mut site_plus_nu = site;
            site_plus_nu[nu] = (site_plus_nu[nu] + 1) % shape[nu];

            let u_nu_at_n_plus_mu = self.get_link_or_identity(&LatticeCell::edge(site_plus_mu, nu));
            let u_mu_at_n_plus_nu = self.get_link_or_identity(&LatticeCell::edge(site_plus_nu, mu));
            let u_nu_at_n = self.get_link_or_identity(&LatticeCell::edge(site, nu));

            let forward = u_nu_at_n_plus_mu
                .mul(&u_mu_at_n_plus_nu.dagger())
                .mul(&u_nu_at_n.dagger());

            // Backward staple: U_ν†(n+μ̂-ν̂) U_μ†(n-ν̂) U_ν(n-ν̂)
            let mut site_minus_nu = site;
            site_minus_nu[nu] = (site_minus_nu[nu] + shape[nu] - 1) % shape[nu];

            let mut site_plus_mu_minus_nu = site_plus_mu;
            site_plus_mu_minus_nu[nu] = (site_plus_mu_minus_nu[nu] + shape[nu] - 1) % shape[nu];

            let u_nu_at_n_plus_mu_minus_nu =
                self.get_link_or_identity(&LatticeCell::edge(site_plus_mu_minus_nu, nu));
            let u_mu_at_n_minus_nu =
                self.get_link_or_identity(&LatticeCell::edge(site_minus_nu, mu));
            let u_nu_at_n_minus_nu =
                self.get_link_or_identity(&LatticeCell::edge(site_minus_nu, nu));

            let backward = u_nu_at_n_plus_mu_minus_nu
                .dagger()
                .mul(&u_mu_at_n_minus_nu.dagger())
                .mul(&u_nu_at_n_minus_nu);

            // Add staples to sum
            staple_sum = staple_sum.add(&forward);
            staple_sum = staple_sum.add(&backward);
        }

        Ok(staple_sum)
    }

    /// Local action change if link U is replaced by U'.
    ///
    /// ΔS = -β Re[Tr((U' - U) · V†)] / N
    ///
    /// where V is the staple sum.
    ///
    /// # Errors
    ///
    /// Returns error if computation fails.
    pub fn try_local_action_change(
        &self,
        edge: &LatticeCell<D>,
        new_link: &LinkVariable<G, T>,
    ) -> Result<T, TopologyError>
    where
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<Output = T>
            + std::ops::Neg<Output = T>
            + From<f64>,
    {
        let old_link = self.get_link_or_identity(edge);
        let staple = self.try_staple(edge)?;

        let n = G::matrix_dim();
        let n_t = T::from(n as f64);

        // ΔS = β * (Re[Tr(U·V†)] - Re[Tr(U'·V†)]) / N
        // (This is the change in action, negative means lower action)
        let staple_dag = staple.dagger();
        let old_tr = old_link.mul(&staple_dag).re_trace();
        let new_tr = new_link.mul(&staple_dag).re_trace();

        Ok(self.beta.clone() * (old_tr - new_tr) / n_t)
    }
}
