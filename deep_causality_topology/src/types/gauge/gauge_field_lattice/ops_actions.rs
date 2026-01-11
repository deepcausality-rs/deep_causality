/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Improved lattice gauge actions.
//!
//! Implements Symanzik-improved actions including Lüscher-Weisz, Iwasaki, and DBW2.
//! These actions reduce discretization errors from $O(a^2)$ to $O(a^4)$ or better.

use crate::{CWComplex, GaugeGroup, LatticeGaugeField, TopologyError};
use deep_causality_num::Float;
use deep_causality_tensor::TensorData;

/// Coefficients for improved gauge actions.
#[derive(Debug, Clone, Copy)]
pub struct ActionCoeffs<T> {
    /// Plaquette (1×1) coefficient c_0.
    pub c0: T,
    /// Rectangle (1×2) coefficient c_1.
    pub c1: T,
}

impl<T> ActionCoeffs<T>
where
    T: Float,
{
    /// Tree-level Symanzik: c_1 = -1/12.
    ///
    /// # Errors
    ///
    /// Returns error if numerical type conversion fails.
    pub fn try_symanzik() -> Result<Self, TopologyError> {
        let c1 = T::from(-1.0 / 12.0).ok_or_else(|| {
            TopologyError::LatticeGaugeError("Failed to convert c1 coefficient to T".to_string())
        })?;
        let c0 = T::from(1.0 + 8.0 / 12.0).ok_or_else(|| {
            TopologyError::LatticeGaugeError("Failed to convert c0 coefficient to T".to_string())
        })?; // c_0 = 1 - 8*c_1
        Ok(Self { c0, c1 })
    }

    /// Iwasaki: c_1 = -0.331.
    ///
    /// # Errors
    ///
    /// Returns error if numerical type conversion fails.
    pub fn try_iwasaki() -> Result<Self, TopologyError> {
        let c1 = T::from(-0.331).ok_or_else(|| {
            TopologyError::LatticeGaugeError("Failed to convert c1 coefficient to T".to_string())
        })?;
        let c0 = T::from(1.0 + 8.0 * 0.331).ok_or_else(|| {
            TopologyError::LatticeGaugeError("Failed to convert c0 coefficient to T".to_string())
        })?;
        Ok(Self { c0, c1 })
    }

    /// DBW2: c_1 = -1.4088.
    ///
    /// # Errors
    ///
    /// Returns error if numerical type conversion fails.
    pub fn try_dbw2() -> Result<Self, TopologyError> {
        let c1 = T::from(-1.4088).ok_or_else(|| {
            TopologyError::LatticeGaugeError("Failed to convert c1 coefficient to T".to_string())
        })?;
        let c0 = T::from(1.0 + 8.0 * 1.4088).ok_or_else(|| {
            TopologyError::LatticeGaugeError("Failed to convert c0 coefficient to T".to_string())
        })?;
        Ok(Self { c0, c1 })
    }

    /// Custom coefficients.
    pub fn custom(c0: T, c1: T) -> Self {
        Self { c0, c1 }
    }
}

impl<G: GaugeGroup, const D: usize, T: TensorData> LatticeGaugeField<G, D, T> {
    /// Compute the Symanzik-improved gauge action.
    ///
    /// # Mathematics
    ///
    /// The improved action includes both 1×1 plaquettes and 1×2 rectangles:
    ///
    /// $$S = \beta \left[ c_0 \sum_{plaq} \left(1 - \frac{1}{N}\text{ReTr}U_{plaq}\right)
    ///                  + c_1 \sum_{rect} \left(1 - \frac{1}{N}\text{ReTr}U_{rect}\right) \right]$$
    ///
    /// Normalization: $c_0 + 8c_1 = 1$ ensures correct continuum limit.
    ///
    /// # Physics
    ///
    /// Improved actions suppress lattice artifacts (scaling violations).
    /// - **Symanzik:** Tree-level improvement ($O(a^2)$ removed)
    /// - **Iwasaki:** Renormalization group improved
    /// - **DBW2:** doubly blocked Wilson 2 (better for thermodynamics)
    ///
    /// # Arguments
    ///
    /// * `coeffs` - Action coefficients ($c_0, c_1$)
    ///
    /// # Returns
    ///
    /// The total action value S.
    ///
    /// # Errors
    ///
    /// Returns error if plaquette/rectangle computation fails.
    pub fn try_improved_action(&self, coeffs: &ActionCoeffs<T>) -> Result<T, TopologyError>
    where
        T: Float,
    {
        let n = G::matrix_dim();
        let n_t = T::from(n as f64).ok_or_else(|| {
            TopologyError::LatticeGaugeError("Failed to convert matrix dimension to T".to_string())
        })?;
        let one = T::one();

        let mut plaq_sum = T::zero();
        let mut rect_sum = T::zero();

        // Plaquettes (1×1) and Rectangles (1×2)
        for site_cell in self.lattice.cells(0) {
            let site = *site_cell.position();
            for mu in 0..D {
                for nu in (mu + 1)..D {
                    // Plaquette contribution
                    let plaq = self.try_plaquette(&site, mu, nu)?;
                    let tr_plaq = plaq.re_trace();
                    let s_plaq = one - tr_plaq / n_t;
                    plaq_sum = plaq_sum + s_plaq;

                    // Rectangle (1×2) contribution - two orientations
                    let rect1 = self.try_rectangle(&site, mu, nu)?;
                    let tr_rect1 = rect1.re_trace();
                    let s_rect1 = one - tr_rect1 / n_t;
                    rect_sum = rect_sum + s_rect1;

                    // Rectangle (2×1) = (1×2) with swapped directions
                    let rect2 = self.try_rectangle(&site, nu, mu)?;
                    let tr_rect2 = rect2.re_trace();
                    let s_rect2 = one - tr_rect2 / n_t;
                    rect_sum = rect_sum + s_rect2;
                }
            }
        }

        // S = β * (c_0 * plaq_sum + c_1 * rect_sum)
        Ok(self.beta * (coeffs.c0 * plaq_sum + coeffs.c1 * rect_sum))
    }
}
