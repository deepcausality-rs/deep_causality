/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// GrOps Implementation for GR (GaugeField<Lorentz, f64, f64>)
// =============================================================================

use crate::{
    GR, GrOps, PhysicsError, einstein_tensor_kernel, geodesic_integrator_kernel,
    parallel_transport_kernel, proper_time_kernel,
};
use deep_causality_metric::LorentzianMetric;
use deep_causality_tensor::CausalTensor;

impl GrOps for GR {
    fn ricci_tensor(&self) -> Result<CausalTensor<f64>, PhysicsError> {
        let riemann = self.field_strength();
        let dim = 4;

        // Use East Coast metric type info for the wrapper (needed for structure, though unused for Ricci contraction)
        let metric_sig = deep_causality_metric::EastCoastMetric::minkowski_4d().into_metric();

        let ct = deep_causality_topology::CurvatureTensor::<(), (), (), ()>::new(
            riemann.clone(),
            metric_sig,
            deep_causality_topology::CurvatureSymmetry::Riemann,
            dim,
        );

        let ricci_data = ct.ricci_tensor();
        Ok(CausalTensor::from_vec(ricci_data, &[dim, dim]))
    }

    fn ricci_scalar(&self) -> Result<f64, PhysicsError> {
        // Use CurvatureTensor for the complex Riemann->Ricci contraction
        let ricci = self.ricci_tensor()?;
        let ricci_data = ricci.as_slice();

        // Use the DYNAMIC metric from the field for the scalar contraction
        // (CurvatureTensor::ricci_scalar assumes static Minkowski metric, which is incorrect for GR)
        let metric = self.metric_tensor();
        let metric_data = metric.as_slice();
        let dim = 4;

        let mut scalar = 0.0;
        for mu in 0..dim {
            for nu in 0..dim {
                // R = g^μν R_μν
                // We need inverse metric g^μν.
                // Assuming diagonal metric for basic compatibility with current tests conventions
                // TODO: Implement full matrix inversion for off-diagonal metrics
                if mu == nu {
                    let g_mu_mu = metric_data.get(mu * dim + mu).copied().unwrap_or(1.0);
                    let g_inv = if g_mu_mu.abs() > 1e-12 {
                        1.0 / g_mu_mu
                    } else {
                        0.0
                    };
                    let r_mu_mu = ricci_data.get(mu * dim + mu).copied().unwrap_or(0.0);
                    scalar += g_inv * r_mu_mu;
                }
            }
        }

        Ok(scalar)
    }

    fn einstein_tensor(&self) -> Result<CausalTensor<f64>, PhysicsError> {
        let ricci = self.ricci_tensor()?;
        let scalar = self.ricci_scalar()?;
        // Delegate to kernel or manual construction
        einstein_tensor_kernel(&ricci, scalar, self.metric_tensor())
    }

    fn kretschmann_scalar(&self) -> Result<f64, PhysicsError> {
        let riemann = self.field_strength();
        let riemann_data = riemann.as_slice();

        // Minimal implementation assuming sum of squares usage (matching CurvatureTensor logic but locally)
        // Note: Full Lorentzian contraction requires inverse metric raising, pending tensor algebra expansion.
        let mut k = 0.0;
        for item in riemann_data {
            k += item * item;
        }

        Ok(k)
    }

    fn geodesic_deviation(
        &self,
        velocity: &[f64],
        separation: &[f64],
    ) -> Result<Vec<f64>, PhysicsError> {
        let riemann = self.field_strength();
        let dim = 4;
        let metric_sig = deep_causality_metric::EastCoastMetric::minkowski_4d().into_metric();

        // Use TensorVector for HKT safety contract
        let u = deep_causality_topology::TensorVector::new(velocity);
        let v = deep_causality_topology::TensorVector::new(separation);
        // Note: u and v are cloned for the calls as HKT consumes them by value in this specific impl
        let u_w = u.clone();

        // Construct CurvatureTensor with correct phantom types for HKT witness
        let ct = deep_causality_topology::CurvatureTensor::<
            deep_causality_topology::TensorVector,
            deep_causality_topology::TensorVector,
            deep_causality_topology::TensorVector,
            deep_causality_topology::TensorVector,
        >::new(
            riemann.clone(),
            metric_sig,
            deep_causality_topology::CurvatureSymmetry::Riemann,
            dim,
        );

        // Use RiemannMap HKT trait via witness
        // D^2 ξ / dτ^2 = R(u, ξ)u
        use deep_causality_haft::RiemannMap;
        use deep_causality_topology::CurvatureTensorWitness;

        let result_vector: deep_causality_topology::TensorVector =
            CurvatureTensorWitness::curvature(ct, u, v, u_w);
        Ok(result_vector.into())
    }

    fn solve_geodesic(
        &self,
        initial_position: &[f64],
        initial_velocity: &[f64],
        proper_time_step: f64,
        num_steps: usize,
    ) -> Result<Vec<crate::theories::gr::GeodesicState>, PhysicsError> {
        geodesic_integrator_kernel(
            initial_position,
            initial_velocity,
            self.connection(),
            proper_time_step,
            num_steps,
        )
    }

    fn proper_time(&self, path: &[Vec<f64>]) -> Result<f64, PhysicsError> {
        proper_time_kernel(path, self.metric_tensor())
    }

    fn parallel_transport(
        &self,
        initial_vector: &[f64],
        path: &[Vec<f64>],
    ) -> Result<Vec<f64>, PhysicsError> {
        parallel_transport_kernel(initial_vector, path, self.connection())
    }

    fn metric_tensor(&self) -> &CausalTensor<f64> {
        self.connection()
    }
}
