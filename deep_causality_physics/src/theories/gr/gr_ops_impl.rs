/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// =============================================================================
// GrOps Implementation for GR (GaugeField<Lorentz, f64, f64>)
// =============================================================================

use crate::theories::gr::gr_utils;
use crate::{
    GR, GeodesicState, GrOps, PhysicsError, einstein_tensor_kernel, geodesic_integrator_kernel,
    parallel_transport_kernel, proper_time_kernel,
};
use deep_causality_haft::RiemannMap;
use deep_causality_metric::{EastCoastMetric, LorentzianMetric};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    CurvatureSymmetry, CurvatureTensor, CurvatureTensorVector, CurvatureTensorWitness, TensorVector,
};

impl GrOps for GR {
    fn ricci_tensor(&self) -> Result<CausalTensor<f64>, PhysicsError> {
        let riemann = self.field_strength();
        let dim = 4;

        // Use East Coast metric type info for the wrapper (needed for structure, though unused for Ricci contraction)
        let metric_sig = EastCoastMetric::minkowski_4d().into_metric();

        let ct = CurvatureTensor::<(), (), (), ()>::new(
            riemann.clone(),
            metric_sig,
            CurvatureSymmetry::Riemann,
            dim,
        );

        let ricci_data = ct.ricci_tensor();
        Ok(CausalTensor::from_vec(ricci_data, &[dim, dim]))
    }

    fn ricci_scalar(&self) -> Result<f64, PhysicsError> {
        // Use CurvatureTensor for the complex Riemann->Ricci contraction
        let ricci = self.ricci_tensor()?;
        let ricci_data = ricci.as_slice();

        // Use the metric from the field for the scalar contraction
        let metric = self.metric_tensor();
        let dim = 4;

        // Full 4x4 Matrix Inversion
        let inv_metric = gr_utils::invert_4x4(metric)?;

        let mut scalar = 0.0;
        // R = g^μν R_μν
        for mu in 0..dim {
            for nu in 0..dim {
                // Flattened index [mu, nu]
                let idx = mu * dim + nu;
                let g_upper = inv_metric[idx];
                let r_lower = ricci_data.get(idx).copied().unwrap_or(0.0);
                scalar += g_upper * r_lower;
            }
        }

        Ok(scalar)
    }

    fn einstein_tensor(&self) -> Result<CausalTensor<f64>, PhysicsError> {
        let ricci = self.ricci_tensor()?;
        let scalar = self.ricci_scalar()?;
        einstein_tensor_kernel(&ricci, scalar, self.metric_tensor())
    }

    fn kretschmann_scalar(&self) -> Result<f64, PhysicsError> {
        let riemann = self.field_strength();
        let r_data = riemann.as_slice(); // R_abcd
        let dim = 4;

        if r_data.len() < dim * dim * dim * dim {
            return Err(PhysicsError::DimensionMismatch(
                "Mapping Lie Algebra curvature to Geometric curvature requires expansion.".into(),
            ));
        }

        // 1. Get Inverse Metric
        let metric = self.metric_tensor();
        let inv_g = gr_utils::invert_4x4(metric)?;

        // Helper to index flat 4D array
        let idx4 = |a, b, c, d| ((a * dim + b) * dim + c) * dim + d;

        // Storage for intermediate raised tensors
        let mut r_raised = r_data.to_vec(); // Start with R_abcd
        let mut temp = vec![0.0; dim * dim * dim * dim];

        // Raise 1st index: R^a_bcd = g^am R_mbcd
        for a in 0..dim {
            for b in 0..dim {
                for c in 0..dim {
                    for d in 0..dim {
                        let mut sum = 0.0;
                        for m in 0..dim {
                            // g^am * R_mbcd
                            sum += inv_g[a * dim + m] * r_raised[idx4(m, b, c, d)];
                        }
                        temp[idx4(a, b, c, d)] = sum;
                    }
                }
            }
        }
        r_raised.copy_from_slice(&temp);

        // Raise 2nd index: R^ab_cd = g^bn R^a_ncd
        for a in 0..dim {
            for b in 0..dim {
                for c in 0..dim {
                    for d in 0..dim {
                        let mut sum = 0.0;
                        for n in 0..dim {
                            sum += inv_g[b * dim + n] * r_raised[idx4(a, n, c, d)];
                        }
                        temp[idx4(a, b, c, d)] = sum;
                    }
                }
            }
        }
        r_raised.copy_from_slice(&temp);

        // Raise 3rd index: R^abc_d = g^cr R^ab_rd
        for a in 0..dim {
            for b in 0..dim {
                for c in 0..dim {
                    for d in 0..dim {
                        let mut sum = 0.0;
                        for r in 0..dim {
                            sum += inv_g[c * dim + r] * r_raised[idx4(a, b, r, d)];
                        }
                        temp[idx4(a, b, c, d)] = sum;
                    }
                }
            }
        }
        r_raised.copy_from_slice(&temp);

        // Raise 4th index: R^abcd = g^ds R^abc_s
        for a in 0..dim {
            for b in 0..dim {
                for c in 0..dim {
                    for d in 0..dim {
                        let mut sum = 0.0;
                        for s in 0..dim {
                            sum += inv_g[d * dim + s] * r_raised[idx4(a, b, c, s)];
                        }
                        temp[idx4(a, b, c, d)] = sum;
                    }
                }
            }
        }
        r_raised.copy_from_slice(&temp); // Now r_raised holds R^abcd

        // Final Contraction: K = R_abcd * R^abcd
        let mut k = 0.0;
        // Verify lengths match before iteration to prevent panic
        if r_data.len() != r_raised.len() {
            return Err(PhysicsError::DimensionMismatch(
                "Riemann tensor data length mismatch".into(),
            ));
        }

        for i in 0..r_data.len() {
            k += r_data[i] * r_raised[i];
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
        let metric_sig = EastCoastMetric::minkowski_4d().into_metric();

        // Use TensorVector for HKT safety contract
        let u = TensorVector::new(velocity);
        let v = TensorVector::new(separation);
        // Note: u and v are cloned for the calls as HKT consumes them by value in this specific impl
        let u_w = u.clone();

        // Construct CurvatureTensorVectorfor HKT witness
        let ct = CurvatureTensorVector::new(
            riemann.clone(),
            metric_sig,
            CurvatureSymmetry::Riemann,
            dim,
        );

        // Use RiemannMap HKT trait via witness type
        // D^2 ξ / dτ^2 = R(u, ξ)u
        let result_vector = CurvatureTensorWitness::curvature(ct, u, v, u_w);
        Ok(result_vector.into())
    }

    fn solve_geodesic(
        &self,
        initial_position: &[f64],
        initial_velocity: &[f64],
        proper_time_step: f64,
        num_steps: usize,
    ) -> Result<Vec<GeodesicState>, PhysicsError> {
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
