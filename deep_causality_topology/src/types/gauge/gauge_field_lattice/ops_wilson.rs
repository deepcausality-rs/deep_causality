/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CWComplex, GaugeGroup, LatticeGaugeField, TopologyError};

// ============================================================================
// Wilson Action
// ============================================================================
impl<G: GaugeGroup, const D: usize, T: Clone + Default> LatticeGaugeField<G, D, T> {
    /// Compute the Wilson action: S = β Σ_p (1 - Re[Tr(U_p)]/N).
    ///
    /// For identity configuration, action = 0.
    ///
    /// # Errors
    ///
    /// Returns error if plaquette computation fails.
    pub fn try_wilson_action(&self) -> Result<T, TopologyError>
    where
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<Output = T>
            + std::ops::Neg<Output = T>
            + From<f64>,
    {
        let n = G::matrix_dim();
        let n_t = T::from(n as f64);
        let one = T::from(1.0);

        let mut action = T::from(0.0);

        // Sum over all sites and all planes μ < ν
        for site_cell in self.lattice.cells(0) {
            let site = *site_cell.position();
            for mu in 0..D {
                for nu in (mu + 1)..D {
                    let plaq = self.try_plaquette(&site, mu, nu)?;
                    let tr = plaq.re_trace();
                    // S_p = 1 - Re[Tr(U_p)] / N
                    let s_p = one.clone() - tr / n_t.clone();
                    action = action + s_p;
                }
            }
        }

        // Multiply by β
        Ok(self.beta.clone() * action)
    }

    /// Action contribution from a single plaquette.
    ///
    /// # Errors
    ///
    /// Returns error if plaquette computation fails.
    pub fn try_plaquette_action(
        &self,
        site: &[usize; D],
        mu: usize,
        nu: usize,
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
        let n = G::matrix_dim();
        let n_t = T::from(n as f64);
        let one = T::from(1.0);

        let plaq = self.try_plaquette(site, mu, nu)?;
        let tr = plaq.re_trace();
        let s_p = one - tr / n_t;

        Ok(self.beta.clone() * s_p)
    }
}
