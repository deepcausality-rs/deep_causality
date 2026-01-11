/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Smearing algorithms for lattice gauge fields.
//!
//! Implements APE (Array Processor Experiment) smearing and Stout smearing
//! to reduce ultraviolet fluctuations and enhance the signal of long-range physics.

use crate::{GaugeGroup, LatticeGaugeField, TopologyError};
use std::collections::HashMap;

// ============================================================================
// Smearing Algorithms
// ============================================================================

/// Smearing parameters.
#[derive(Debug, Clone)]
pub struct SmearingParams<T> {
    /// Smearing weight α (APE/HYP) or ρ (stout).
    pub alpha: T,
    /// Number of smearing iterations.
    pub n_steps: usize,
}

impl<T: From<f64>> SmearingParams<T> {
    /// Default APE smearing parameters.
    pub fn ape_default() -> Self {
        Self {
            alpha: T::from(0.45),
            n_steps: 10,
        }
    }

    /// Default stout smearing parameters.
    pub fn stout_default() -> Self {
        Self {
            alpha: T::from(0.12),
            n_steps: 6,
        }
    }
}

impl<G: GaugeGroup, const D: usize, T: Clone + Default> LatticeGaugeField<G, D, T> {
    /// Apply APE smearing to the gauge field.
    ///
    /// # Mathematics
    ///
    /// APE smearing replaces each link $U_\mu(x)$ with a projected average of itself
    /// and its staples:
    ///
    /// $$U'_\mu(x) = \text{Proj}_{SU(N)}\left[ (1-\alpha) U_\mu(x) + \frac{\alpha}{2(D-1)} \sum_{\nu \neq \mu} S_{\mu\nu}(x) \right]$$
    ///
    /// where $S_{\mu\nu}$ is the sum of forward and backward staples in the $\mu-\nu$ plane.
    ///
    /// # Physics
    ///
    /// Smearing suppresses UV fluctuations (short-distance noise) while preserving
    /// the long-distance physical content (IR properties). It is essential for:
    /// - Improving signal-to-noise ratio in glueball/loop operators
    /// - Defining topological charge on the lattice
    ///
    /// # Arguments
    ///
    /// * `params` - Smearing parameters ($\alpha$, number of steps)
    ///
    /// # Returns
    ///
    /// A new `LatticeGaugeField` with smeared links.
    ///
    /// # Errors
    ///
    /// Returns error if smearing computation fails.
    pub fn try_smear(&self, params: &SmearingParams<T>) -> Result<Self, TopologyError>
    where
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<Output = T>
            + std::ops::Neg<Output = T>
            + From<f64>
            + PartialOrd,
    {
        let mut current = self.clone();
        let alpha = params.alpha.clone();
        let one_minus_alpha = T::from(1.0) - alpha.clone();
        let staple_weight = alpha / T::from(2.0 * (D - 1) as f64);

        for _step in 0..params.n_steps {
            let mut new_links = HashMap::new();

            for (edge, old_link) in current.links.iter() {
                // Compute staple sum
                let staple = current.try_staple(edge)?;

                // Weighted combination: (1-α) U + (α/(2(D-1))) V
                let weighted_old = old_link.scale(&one_minus_alpha);
                let weighted_staple = staple.scale(&staple_weight);
                let combined = weighted_old.add(&weighted_staple);

                // Project to SU(N)
                let projected = combined.project_sun().map_err(TopologyError::from)?;

                new_links.insert(edge.clone(), projected);
            }

            current.links = new_links;
        }

        Ok(current)
    }
}
