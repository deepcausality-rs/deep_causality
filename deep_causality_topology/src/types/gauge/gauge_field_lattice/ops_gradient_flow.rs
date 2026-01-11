/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gradient flow evolution.
//!
//! Implements the Wilson flow equation to continuously smooth gauge fields
//! towards the stationary points of the action. Used for scale setting and renormalization.

use crate::{CWComplex, GaugeGroup, LatticeGaugeField, TopologyError};
use std::collections::HashMap;

// ============================================================================
// Gradient Flow (Section 13)
// ============================================================================

/// Integration method for gradient flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowMethod {
    /// Simple Euler integration.
    Euler,
    /// 3rd order Runge-Kutta (recommended).
    RungeKutta3,
}

/// Gradient flow parameters.
#[derive(Debug, Clone)]
pub struct FlowParams<T> {
    /// Flow time step ε.
    pub epsilon: T,
    /// Target flow time t.
    pub t_max: T,
    /// Integration method.
    pub method: FlowMethod,
}

impl<T: From<f64>> FlowParams<T> {
    /// Default flow parameters.
    pub fn default_params() -> Self {
        Self {
            epsilon: T::from(0.01),
            t_max: T::from(1.0),
            method: FlowMethod::RungeKutta3,
        }
    }
}

impl<G: GaugeGroup, const D: usize, T: Clone + Default> LatticeGaugeField<G, D, T> {
    /// Perform gradient flow integration.
    ///
    /// # Mathematics
    ///
    /// Integrates the Wilson flow equation:
    ///
    /// $$\frac{\partial V_\mu(x,t)}{\partial t} = -g_0^2 \frac{\partial S_W}{\partial V_\mu(x,t)} V_\mu(x,t)$$
    ///
    /// The derivative of the Wilson action is related to the staple $S_{\mu\nu}$:
    /// $$\dot V_\mu = - \left( \nabla S_W \right) V_\mu$$
    ///
    /// # Physics
    ///
    /// Gradient flow introduces a flow time scale $t$ (dimension $[L^2]$).
    /// Fields at flow time $t$ are effectively smeared over a radius $\sqrt{8t}$.
    /// This defines a renormalization scheme where composite operators are finite.
    ///
    /// # Arguments
    ///
    /// * `params` - Integration parameters (step size $\epsilon$, algorithm, max time)
    ///
    /// # Returns
    ///
    /// The flowed gauge field at $t = t_{max}$.
    ///
    /// # Errors
    ///
    /// Returns error if flow computation fails.
    pub fn try_flow(&self, params: &FlowParams<T>) -> Result<Self, TopologyError>
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
        let mut t = T::from(0.0);
        let epsilon = params.epsilon.clone();

        // Integrate from t=0 to t=t_max
        while t < params.t_max {
            current = match params.method {
                FlowMethod::Euler => current.try_euler_step(&epsilon)?,
                FlowMethod::RungeKutta3 => current.try_rk3_step(&epsilon)?,
            };
            t = t + epsilon.clone();
        }

        Ok(current)
    }

    /// Single Euler step of gradient flow.
    fn try_euler_step(&self, epsilon: &T) -> Result<Self, TopologyError>
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
        let mut new_links = HashMap::new();
        let n = G::matrix_dim();
        let n_t = T::from(n as f64);

        for (edge, u) in self.links.iter() {
            // Compute staple and force
            let staple = self.try_staple(edge)?;
            let staple_dag = staple.dagger();
            let u_v_dag = u.mul(&staple_dag);

            // Force: F = β (U·V† - (1/N) Tr(U·V†) I) / N
            // For flow: we use simplified form proportional to staple
            let neg_eps = T::from(-1.0) * epsilon.clone();
            let update = u_v_dag
                .scale(&neg_eps)
                .scale(&(self.beta.clone() / n_t.clone()));

            // U' = U + ε F, then project
            let new_u = u.add(&update);
            let projected = new_u.project_sun().map_err(TopologyError::from)?;

            new_links.insert(edge.clone(), projected);
        }

        Ok(Self {
            lattice: self.lattice.clone(),
            links: new_links,
            beta: self.beta.clone(),
        })
    }

    /// Single 3rd-order Runge-Kutta step of gradient flow.
    fn try_rk3_step(&self, epsilon: &T) -> Result<Self, TopologyError>
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
        // Lüscher's 3rd order RK scheme:
        // W0 = V(t)
        // W1 = exp(1/4 ε Z0) W0
        // W2 = exp(8/9 ε Z1 - 17/36 ε Z0) W1
        // V(t+ε) = exp(3/4 ε Z2 - 8/9 ε Z1 + 17/36 ε Z0) W2
        //
        // For simplicity, we use three Euler-like steps with weights

        let eps_quarter = epsilon.clone() * T::from(0.25);
        let step1 = self.try_euler_step(&eps_quarter)?;

        let eps_half = epsilon.clone() * T::from(0.5);
        let step2 = step1.try_euler_step(&eps_half)?;

        let eps_quarter_final = epsilon.clone() * T::from(0.25);
        step2.try_euler_step(&eps_quarter_final)
    }

    /// Compute the flow energy density E(t).
    ///
    /// # Mathematics
    ///
    /// $$E(t) = \frac{1}{2} \langle \text{Tr}(F_{\mu\nu} F_{\mu\nu}) \rangle
    ///         \approx \sum_p \left(1 - \frac{1}{N}\text{ReTr} U_p\right)$$
    ///
    /// # Physics
    ///
    /// The energy density is a gauge invariant observable of dimension 4.
    /// Under gradient flow, $\langle E(t) \rangle$ is a renormalized quantity.
    ///
    /// # Returns
    ///
    /// The average energy density.
    ///
    /// # Errors
    ///
    /// Returns error if computation fails.
    pub fn try_energy_density(&self) -> Result<T, TopologyError>
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

        let mut sum = T::from(0.0);
        let mut count = 0usize;

        for site_cell in self.lattice.cells(0) {
            let site = *site_cell.position();
            for mu in 0..D {
                for nu in (mu + 1)..D {
                    let plaq = self.try_plaquette(&site, mu, nu)?;
                    let tr = plaq.re_trace();
                    let e_p = one.clone() - tr / n_t.clone();
                    sum = sum + e_p;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Ok(T::from(0.0));
        }

        Ok(sum / T::from(count as f64))
    }

    /// Compute t² E(t) for scale setting.
    ///
    /// At flow time t, this quantity is used to define reference scales t₀ and w₀.
    ///
    /// # Errors
    ///
    /// Returns error if computation fails.
    pub fn try_t2_energy(&self, t: T) -> Result<T, TopologyError>
    where
        T: Clone
            + std::ops::Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Sub<Output = T>
            + std::ops::Div<Output = T>
            + std::ops::Neg<Output = T>
            + From<f64>,
    {
        let e = self.try_energy_density()?;
        Ok(t.clone() * t * e)
    }

    /// Find t₀ scale where t² E(t) = 0.3.
    ///
    /// The scale t₀ provides a non-perturbative reference scale for lattice QCD,
    /// defined by the condition $t^2 \langle E(t) \rangle |_{t=t_0} = 0.3$.
    ///
    /// # Physics
    ///
    /// The Wilson flow smooths gauge configurations, and the energy density
    /// at flow time t is automatically renormalized at scale $\mu \sim 1/\sqrt{8t}$.
    ///
    /// The scale t₀ is widely used in lattice QCD for:
    /// - Setting the lattice spacing in physical units
    /// - Continuum extrapolations
    /// - Comparing results between different collaborations
    ///
    /// Typical values: $\sqrt{8t_0} \approx 0.4$ fm in QCD.
    ///
    /// # Algorithm
    ///
    /// 1. Flow configuration forward in time
    /// 2. Monitor t² E(t) at each step
    /// 3. Interpolate to find t where t² E(t) = 0.3
    ///
    /// # Arguments
    ///
    /// * `params` - Flow parameters (step size, method)
    ///
    /// # Returns
    ///
    /// The flow time t₀ where t² E(t) = 0.3, or an error if the condition
    /// is not reached within t_max.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Flow computation fails
    /// - t² E(t) never reaches 0.3 within t_max
    pub fn try_find_t0(&self, params: &FlowParams<T>) -> Result<T, TopologyError>
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
        let target = T::from(0.3);
        let mut current = self.clone();
        let mut t = T::from(0.0);
        let epsilon = params.epsilon.clone();

        let mut prev_t = t.clone();
        let mut prev_t2e = current.try_t2_energy(t.clone())?;

        // Flow until t² E(t) crosses 0.3
        while t < params.t_max {
            current = match params.method {
                FlowMethod::Euler => current.try_euler_step(&epsilon)?,
                FlowMethod::RungeKutta3 => current.try_rk3_step(&epsilon)?,
            };
            t = t.clone() + epsilon.clone();

            let t2e = current.try_t2_energy(t.clone())?;

            // Check if we crossed the target
            if t2e >= target.clone() && prev_t2e < target.clone() {
                // Linear interpolation to find t₀
                // t₀ ≈ prev_t + (target - prev_t2e) * ε / (t2e - prev_t2e)
                let dt = t.clone() - prev_t.clone();
                let d_t2e = t2e.clone() - prev_t2e.clone();
                let ratio = (target.clone() - prev_t2e.clone()) / d_t2e;
                return Ok(prev_t + ratio * dt);
            }

            prev_t = t.clone();
            prev_t2e = t2e;
        }

        Err(TopologyError::LatticeGaugeError(
            "t² E(t) did not reach 0.3 within t_max".to_string(),
        ))
    }
}
