/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CWComplex, GaugeGroup, LatticeCell, LatticeGaugeField, LinkVariable, TopologyError};

impl<G: GaugeGroup, const D: usize, T: Clone + Default> LatticeGaugeField<G, D, T> {
    /// Compute the plaquette U_μν(n) at a given site in a given plane.
    ///
    /// Returns the ordered product of 4 links around the elementary square:
    /// U_μν(n) = U_μ(n) U_ν(n+μ̂) U_μ†(n+ν̂) U_ν†(n)
    ///
    /// # Arguments
    /// * `site` - Base vertex position [x₀, x₁, ..., x_{D-1}]
    /// * `mu` - First direction (0 to D-1)
    /// * `nu` - Second direction (0 to D-1), must differ from mu
    ///
    /// # Errors
    ///
    /// Returns error if directions are invalid.
    pub fn try_plaquette(
        &self,
        site: &[usize; D],
        mu: usize,
        nu: usize,
    ) -> Result<LinkVariable<G, T>, TopologyError>
    where
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Neg<Output = T>
            + From<f64>,
    {
        if mu >= D || nu >= D || mu == nu {
            return Err(TopologyError::LatticeGaugeError(format!(
                "Invalid plaquette directions: mu={}, nu={}, D={}",
                mu, nu, D
            )));
        }

        let shape = self.lattice.shape();

        // Edge 1: n → n + μ̂
        let edge1 = LatticeCell::edge(*site, mu);

        // Edge 2: n + μ̂ → n + μ̂ + ν̂
        let mut site_plus_mu = *site;
        site_plus_mu[mu] = (site_plus_mu[mu] + 1) % shape[mu];
        let edge2 = LatticeCell::edge(site_plus_mu, nu);

        // Edge 3: n + ν̂ → n + μ̂ + ν̂ (traversed backwards)
        let mut site_plus_nu = *site;
        site_plus_nu[nu] = (site_plus_nu[nu] + 1) % shape[nu];
        let edge3 = LatticeCell::edge(site_plus_nu, mu);

        // Edge 4: n → n + ν̂ (traversed backwards)
        let edge4 = LatticeCell::edge(*site, nu);

        // Get link variables
        let u1 = self.get_link_or_identity(&edge1);
        let u2 = self.get_link_or_identity(&edge2);
        let u3 = self.get_link_or_identity(&edge3);
        let u4 = self.get_link_or_identity(&edge4);

        // Plaquette = U_μ(n) U_ν(n+μ̂) U_μ†(n+ν̂) U_ν†(n)
        let u1_u2 = u1.mul(&u2);
        let u3_dag = u3.dagger();
        let u4_dag = u4.dagger();
        let u1_u2_u3dag = u1_u2.mul(&u3_dag);
        let result = u1_u2_u3dag.mul(&u4_dag);

        Ok(result)
    }

    /// Compute the 1×2 rectangle Wilson loop at a given site.
    ///
    /// The rectangle extends 1 unit in direction μ and 2 units in direction ν.
    ///
    /// # Errors
    ///
    /// Returns error if directions are invalid.
    pub fn try_rectangle(
        &self,
        site: &[usize; D],
        mu: usize,
        nu: usize,
    ) -> Result<LinkVariable<G, T>, TopologyError>
    where
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Neg<Output = T>
            + From<f64>,
    {
        if mu >= D || nu >= D || mu == nu {
            return Err(TopologyError::LatticeGaugeError(format!(
                "Invalid rectangle directions: mu={}, nu={}, D={}",
                mu, nu, D
            )));
        }

        let shape = self.lattice.shape();

        // Path: n → n+μ̂ → n+μ̂+ν̂ → n+μ̂+2ν̂ → n+2ν̂ → n+ν̂ → n
        // Forward: U_μ(n), U_ν(n+μ̂), U_ν(n+μ̂+ν̂)
        // Backward: U_μ†(n+2ν̂), U_ν†(n+ν̂), U_ν†(n)

        let mut pos = *site;
        let mut result = LinkVariable::<G, T>::identity();

        // Step 1: n → n+μ̂
        let edge1 = LatticeCell::edge(pos, mu);
        result = result.mul(&self.get_link_or_identity(&edge1));
        pos[mu] = (pos[mu] + 1) % shape[mu];

        // Step 2: n+μ̂ → n+μ̂+ν̂
        let edge2 = LatticeCell::edge(pos, nu);
        result = result.mul(&self.get_link_or_identity(&edge2));
        pos[nu] = (pos[nu] + 1) % shape[nu];

        // Step 3: n+μ̂+ν̂ → n+μ̂+2ν̂
        let edge3 = LatticeCell::edge(pos, nu);
        result = result.mul(&self.get_link_or_identity(&edge3));
        pos[nu] = (pos[nu] + 1) % shape[nu];

        // Step 4: n+μ̂+2ν̂ → n+2ν̂ (backward in μ)
        pos[mu] = (pos[mu] + shape[mu] - 1) % shape[mu];
        let edge4 = LatticeCell::edge(pos, mu);
        result = result.mul(&self.get_link_or_identity(&edge4).dagger());

        // Step 5: n+2ν̂ → n+ν̂ (backward in ν)
        pos[nu] = (pos[nu] + shape[nu] - 1) % shape[nu];
        let edge5 = LatticeCell::edge(pos, nu);
        result = result.mul(&self.get_link_or_identity(&edge5).dagger());

        // Step 6: n+ν̂ → n (backward in ν)
        pos[nu] = (pos[nu] + shape[nu] - 1) % shape[nu];
        let edge6 = LatticeCell::edge(pos, nu);
        result = result.mul(&self.get_link_or_identity(&edge6).dagger());

        Ok(result)
    }

    /// Average plaquette value: (1/N_p) Σ_p Re[Tr(U_p)] / N.
    ///
    /// This is related to the action density.
    /// For identity configuration, returns 1.0.
    /// For random configuration, approaches 0.0.
    ///
    /// # Errors
    ///
    /// Returns error if plaquette computation fails.
    pub fn try_average_plaquette(&self) -> Result<T, TopologyError>
    where
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Div<Output = T>
            + std::ops::Neg<Output = T>
            + From<f64>,
    {
        let n = G::matrix_dim();
        let n_t = T::from(n as f64);

        let mut sum = T::from(0.0);
        let mut count = 0usize;

        // Sum over all sites and all planes μ < ν
        for site_cell in self.lattice.cells(0) {
            let site = *site_cell.position();
            for mu in 0..D {
                for nu in (mu + 1)..D {
                    let plaq = self.try_plaquette(&site, mu, nu)?;
                    let tr = plaq.re_trace();
                    sum = sum + tr;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Ok(T::from(1.0));
        }

        // Average = sum / (count * N)
        let count_t = T::from(count as f64);
        Ok(sum / (count_t * n_t))
    }
}
