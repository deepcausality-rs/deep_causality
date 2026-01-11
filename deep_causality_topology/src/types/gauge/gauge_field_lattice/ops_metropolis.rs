/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Metropolis algorithm for Monte Carlo gauge field updates.
//!
//! The Metropolis algorithm is a Markov chain Monte Carlo method for
//! importance sampling gauge configurations according to the Boltzmann weight.

use crate::{GaugeGroup, LatticeCell, LatticeGaugeField, LinkVariable, TopologyError};
use deep_causality_tensor::TensorData;

// ============================================================================
// Metropolis Updates
// ============================================================================

impl<G: GaugeGroup, const D: usize, T: TensorData> LatticeGaugeField<G, D, T> {
    /// Perform a single Metropolis update on a link.
    ///
    /// Proposes a random modification to link U and accepts or rejects
    /// based on the Metropolis criterion.
    ///
    /// # Algorithm
    ///
    /// 1. Propose: U' = R · U where R is a random SU(N) element near identity
    /// 2. Compute: ΔS = S[U'] - S[U] using the local action change
    /// 3. Accept with probability: min(1, e^{-ΔS})
    ///
    /// # Mathematics
    ///
    /// The Metropolis algorithm satisfies detailed balance:
    ///
    /// $$P[U] \cdot T(U \to U') = P[U'] \cdot T(U' \to U)$$
    ///
    /// where $P[U] \propto e^{-S[U]}$ is the Boltzmann distribution.
    ///
    /// # Arguments
    ///
    /// * `edge` - The link (1-cell) to update
    /// * `epsilon` - Proposal width (smaller = higher acceptance, slower decorrelation)
    /// * `rng` - Random number generator
    ///
    /// # Returns
    ///
    /// `Ok(true)` if update was accepted, `Ok(false)` if rejected.
    ///
    /// # Errors
    ///
    /// Returns error if staple computation or link creation fails.
    pub fn try_metropolis_update<R>(
        &mut self,
        edge: &LatticeCell<D>,
        epsilon: T,
        rng: &mut R,
    ) -> Result<bool, TopologyError>
    where
        R: deep_causality_rand::Rng,
        T: From<f64> + PartialOrd + std::fmt::Debug,
    {
        // Get current link
        let current = self.get_link_or_identity(edge);

        // Generate a small random perturbation
        let perturbation = self.generate_small_su_n_update(epsilon, rng)?;

        // Propose: U' = R · U
        let proposed = perturbation.mul(&current);

        // Compute action change (negative means lower action = favorable)
        let delta_s = self.try_local_action_change(edge, &proposed)?;

        // Metropolis accept/reject
        let accept = if delta_s < T::from(0.0) {
            // Always accept if action decreases
            true
        } else {
            // Accept with probability exp(-ΔS)
            let r: f64 = rng.random();
            // Use debug formatting as a robust fallback for conversion if direct cast isn't available
            // This works for f64, f32, and DoubleFloat (which implements Debug/Display)
            let delta_s_f64 = format!("{:?}", delta_s)
                .parse::<f64>()
                .unwrap_or(f64::INFINITY); // Fail safe to reject if parsing fails

            if delta_s_f64.is_nan() {
                false // Reject NaN actions
            } else {
                r < (-delta_s_f64).exp()
            }
        };

        if accept {
            self.set_link(edge.clone(), proposed);
        }

        Ok(accept)
    }

    /// Perform a full Metropolis sweep over all links.
    ///
    /// Updates each link once in a sequential sweep. The order is determined
    /// by the lattice cell iteration order.
    ///
    /// # Physics
    ///
    /// A single sweep updates each of the D × V links once (V = lattice volume).
    /// Multiple sweeps are needed for thermalization and to reduce autocorrelation.
    ///
    /// Typical simulation structure:
    /// 1. Thermalization: 100-1000 sweeps (discard)
    /// 2. Measurements: Every N sweeps to reduce autocorrelation
    ///
    /// # Arguments
    ///
    /// * `epsilon` - Proposal width (tune for ~50% acceptance rate)
    /// * `rng` - Random number generator
    ///
    /// # Returns
    ///
    /// The acceptance rate (fraction of accepted updates).
    ///
    /// # Errors
    ///
    /// Returns error if any update fails.
    pub fn try_metropolis_sweep<R>(&mut self, epsilon: T, rng: &mut R) -> Result<f64, TopologyError>
    where
        R: deep_causality_rand::Rng,
        T: From<f64> + PartialOrd + std::fmt::Debug,
    {
        let edges: Vec<_> = self.links.keys().cloned().collect();
        let total = edges.len();

        if total == 0 {
            return Ok(0.0);
        }

        let mut accepted = 0usize;

        for edge in edges {
            if self.try_metropolis_update(&edge, epsilon, rng)? {
                accepted += 1;
            }
        }

        Ok(accepted as f64 / total as f64)
    }

    /// Generate a small SU(N) element near identity for Metropolis proposals.
    ///
    /// Creates R ≈ 𝟙 + ε·X where X is a random traceless Hermitian matrix.
    fn generate_small_su_n_update<R>(
        &self,
        epsilon: T,
        rng: &mut R,
    ) -> Result<LinkVariable<G, T>, TopologyError>
    where
        R: deep_causality_rand::Rng,
        T: From<f64> + PartialOrd,
    {
        // Start with identity
        let result = LinkVariable::<G, T>::try_identity().map_err(TopologyError::from)?;

        // Add small random perturbation
        let n = G::matrix_dim();
        let data = result.as_slice();
        let mut new_data = data.to_vec();

        for i in 0..n {
            for j in 0..n {
                let r: f64 = rng.random();
                let perturbation = epsilon * T::from(2.0 * r - 1.0);
                new_data[i * n + j] = new_data[i * n + j] + perturbation;
            }
        }

        // Create perturbed matrix and project to SU(N)
        let tensor = deep_causality_tensor::CausalTensor::new(new_data, vec![n, n])
            .map_err(|e| TopologyError::LatticeGaugeError(format!("{:?}", e)))?;

        let perturbed = LinkVariable::from_matrix_unchecked(tensor);
        perturbed.project_sun().map_err(TopologyError::from)
    }
}

// Helper for f64 conversion when T: Into<f64>
impl<G: GaugeGroup, const D: usize> LatticeGaugeField<G, D, f64> {
    /// Metropolis update specialized for f64.
    ///
    /// Significantly faster than the generic implementation when T=f64.
    ///
    /// # Arguments
    ///
    /// * `edge` - The link to update
    /// * `epsilon` - Proposal step size
    /// * `rng` - Random number generator
    ///
    /// # Returns
    ///
    /// `Ok(true)` if accepted, `Ok(false)` if rejected.
    ///
    /// # Errors
    ///
    /// Returns error if update fails.
    pub fn metropolis_update_f64<R>(
        &mut self,
        edge: &LatticeCell<D>,
        epsilon: f64,
        rng: &mut R,
    ) -> Result<bool, TopologyError>
    where
        R: deep_causality_rand::Rng,
    {
        let current = self.get_link_or_identity(edge);
        let perturbation = self.generate_small_su_n_update(epsilon, rng)?;
        let proposed = perturbation.mul(&current);
        let delta_s = self.try_local_action_change(edge, &proposed)?;

        let accept = if delta_s < 0.0 {
            true
        } else {
            let r: f64 = rng.random();
            r < (-delta_s).exp()
        };

        if accept {
            self.set_link(edge.clone(), proposed);
        }

        Ok(accept)
    }

    /// Full sweep specialized for f64.
    ///
    /// # Arguments
    ///
    /// * `epsilon` - Proposal width
    /// * `rng` - Random number generator
    ///
    /// # Returns
    ///
    /// The acceptance rate (0.0 to 1.0).
    ///
    /// # Errors
    ///
    /// Returns error if any update fails.
    pub fn metropolis_sweep_f64<R>(
        &mut self,
        epsilon: f64,
        rng: &mut R,
    ) -> Result<f64, TopologyError>
    where
        R: deep_causality_rand::Rng,
    {
        let edges: Vec<_> = self.links.keys().cloned().collect();
        let total = edges.len();

        if total == 0 {
            return Ok(0.0);
        }

        let mut accepted = 0usize;

        for edge in edges {
            if self.metropolis_update_f64(&edge, epsilon, rng)? {
                accepted += 1;
            }
        }

        Ok(accepted as f64 / total as f64)
    }
}
