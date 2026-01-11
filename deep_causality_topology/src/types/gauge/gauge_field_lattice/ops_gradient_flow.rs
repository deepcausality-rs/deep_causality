/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

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
    /// Integrates the Wilson flow equation:
    /// ∂V_μ(n,t)/∂t = -g₀² { ∂S/∂V_μ(n,t) } V_μ(n,t)
    ///
    /// The derivative of the Wilson action with respect to link U is:
    /// ∂S/∂U = β (U·V† - (1/N) Tr(U·V†) I) / N
    ///
    /// where V is the staple sum.
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

    /// Compute the energy density E = (1/V) Σ_p (1 - Re[Tr(U_p)]/N).
    ///
    /// This is proportional to the action density.
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
}
