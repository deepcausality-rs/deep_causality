/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CWComplex, GaugeGroup, LatticeGaugeField, TopologyError};

/// Coefficients for improved gauge actions.
#[derive(Debug, Clone, Copy)]
pub struct ActionCoeffs<T> {
    /// Plaquette (1×1) coefficient c_0.
    pub c0: T,
    /// Rectangle (1×2) coefficient c_1.
    pub c1: T,
}

impl<T: From<f64> + Clone> ActionCoeffs<T> {
    /// Tree-level Symanzik: c_1 = -1/12.
    pub fn symanzik() -> Self {
        let c1 = T::from(-1.0 / 12.0);
        let c0 = T::from(1.0 + 8.0 / 12.0); // c_0 = 1 - 8*c_1
        Self { c0, c1 }
    }

    /// Iwasaki: c_1 = -0.331.
    pub fn iwasaki() -> Self {
        let c1 = T::from(-0.331);
        let c0 = T::from(1.0 + 8.0 * 0.331);
        Self { c0, c1 }
    }

    /// DBW2: c_1 = -1.4088.
    pub fn dbw2() -> Self {
        let c1 = T::from(-1.4088);
        let c0 = T::from(1.0 + 8.0 * 1.4088);
        Self { c0, c1 }
    }

    /// Custom coefficients.
    pub fn custom(c0: T, c1: T) -> Self {
        Self { c0, c1 }
    }
}

impl<G: GaugeGroup, const D: usize, T: Clone + Default> LatticeGaugeField<G, D, T> {
    /// Compute Symanzik-improved action with given coefficients.
    ///
    /// S = β [ c_0 Σ_□ (1 - Re[Tr(U_□)]/N) + c_1 Σ_⊟ (1 - Re[Tr(U_⊟)]/N) ]
    ///
    /// # Errors
    ///
    /// Returns error if plaquette/rectangle computation fails.
    pub fn try_improved_action(&self, coeffs: &ActionCoeffs<T>) -> Result<T, TopologyError>
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

        let mut plaq_sum = T::from(0.0);
        let mut rect_sum = T::from(0.0);

        // Plaquettes (1×1) and Rectangles (1×2)
        for site_cell in self.lattice.cells(0) {
            let site = *site_cell.position();
            for mu in 0..D {
                for nu in (mu + 1)..D {
                    // Plaquette contribution
                    let plaq = self.try_plaquette(&site, mu, nu)?;
                    let tr_plaq = plaq.re_trace();
                    let s_plaq = one.clone() - tr_plaq / n_t.clone();
                    plaq_sum = plaq_sum + s_plaq;

                    // Rectangle (1×2) contribution - two orientations
                    let rect1 = self.try_rectangle(&site, mu, nu)?;
                    let tr_rect1 = rect1.re_trace();
                    let s_rect1 = one.clone() - tr_rect1 / n_t.clone();
                    rect_sum = rect_sum + s_rect1;

                    // Rectangle (2×1) = (1×2) with swapped directions
                    let rect2 = self.try_rectangle(&site, nu, mu)?;
                    let tr_rect2 = rect2.re_trace();
                    let s_rect2 = one.clone() - tr_rect2 / n_t.clone();
                    rect_sum = rect_sum + s_rect2;
                }
            }
        }

        // S = β * (c_0 * plaq_sum + c_1 * rect_sum)
        Ok(self.beta.clone() * (coeffs.c0.clone() * plaq_sum + coeffs.c1.clone() * rect_sum))
    }
}
