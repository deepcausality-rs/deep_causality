/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **MMS-verification** workflow (`CfdFlow::verify`): feed a [`Manufactured`] solution's exact
//! pointwise inputs to the regime kernel, check the residual against the exact `∂u/∂t`, and
//! optionally run a kernel-in-the-loop amplitude march against the analytic decay. Pointwise — no
//! geometry, no DEC march.

use crate::theories::incompressible_ns_rhs;
use crate::types::CfdScalar;
use crate::types::flow::{CfdFlow, Report};
use crate::types::flow_config::{Manufactured, VerifyConfig};
use deep_causality_calculus::{EndoArrow, Rk4};
use deep_causality_physics::{
    AccelerationVector, Density, KinematicViscosity, PhysicsError, Velocity3, VelocityGradient,
};

impl CfdFlow {
    /// Compose an MMS-verification workflow from a [`VerifyConfig`].
    pub fn verify<R: CfdScalar, M: Manufactured<R>>(
        config: &VerifyConfig<R, M>,
    ) -> VerifyRun<'_, R, M> {
        VerifyRun { config }
    }
}

/// A runnable MMS-verification workflow.
pub struct VerifyRun<'c, R: CfdScalar, M: Manufactured<R>> {
    config: &'c VerifyConfig<R, M>,
}

impl<R: CfdScalar, M: Manufactured<R>> VerifyRun<'_, R, M> {
    /// Run the verification: kernel residual at the sample point (+ optional amplitude march).
    ///
    /// The report carries `velocity`, `kernel_dudt`, `exact_dudt`, `mms_error`, and — when an
    /// amplitude march is configured — `amplitude_final` and `amplitude_exact`.
    ///
    /// # Errors
    /// Any failure constructing the pointwise quantities or evaluating the regime kernel.
    pub fn run(self) -> Result<Report<R>, PhysicsError> {
        let config = self.config;
        let sample = config.manufactured.sample(&config.point, config.t);
        let rho = config.manufactured.density();
        let nu = config.manufactured.viscosity();
        let lift = |x: f64| R::from_f64(x).expect("a manufactured spec lifts into R");

        let u = Velocity3::<R>::new(sample.velocity)?;
        let grad_u = VelocityGradient::<R>::new(sample.velocity_jacobian)?;
        let rho_q = Density::<R>::new(lift(rho))?;
        let nu_q = KinematicViscosity::<R>::new(lift(nu))?;
        let body = AccelerationVector::<R>::new([R::zero(); 3])?;

        let kernel = incompressible_ns_rhs(
            &u,
            &grad_u,
            &sample.velocity_laplacian,
            &sample.pressure_gradient,
            &rho_q,
            &nu_q,
            &body,
        )?
        .into_inner();
        let kernel_err = max_abs_diff(&kernel, &sample.exact_time_derivative);

        let mut report = Report::new(config.name.clone());
        report.add_series("velocity", sample.velocity.to_vec());
        report.add_series("kernel_dudt", kernel.to_vec());
        report.add_series("exact_dudt", sample.exact_time_derivative.to_vec());
        report.add_series("mms_error", vec![kernel_err]);

        if let Some((dt, steps)) = config.amplitude_march {
            // The Taylor–Green field keeps its spatial shape, so velocity, ∇u and ∇²u scale with the
            // amplitude `a` and pressure with `a²`; the rate runs the same kernel at every step, then
            // projects the acceleration onto the initial velocity to recover da/dt.
            let u0 = sample.velocity;
            let grad_u0 = sample.velocity_jacobian;
            let lap0 = sample.velocity_laplacian;
            let grad_p0 = sample.pressure_gradient;
            let rho_r = Density::<R>::new_unchecked(lift(rho));
            let nu_r = KinematicViscosity::<R>::new_unchecked(lift(nu));
            let body_r = AccelerationVector::<R>::new_unchecked([R::zero(); 3]);
            let u0_sq = u0[0] * u0[0] + u0[1] * u0[1] + u0[2] * u0[2];

            let rate = move |a: &R| {
                let a = *a;
                let uu = Velocity3::<R>::new_unchecked([a * u0[0], a * u0[1], a * u0[2]]);
                let gu = VelocityGradient::<R>::new_unchecked(scale_3x3(grad_u0, a));
                let lap = scale_3(lap0, a);
                let gp = scale_3(grad_p0, a * a);
                let accel = incompressible_ns_rhs(&uu, &gu, &lap, &gp, &rho_r, &nu_r, &body_r)
                    .expect("kernel evaluates")
                    .into_inner();
                (accel[0] * u0[0] + accel[1] * u0[1] + accel[2] * u0[2]) / u0_sq
            };

            let a_final = Rk4::new(dt, rate).iterate_n(R::one(), steps);
            let t_final = dt * lift(steps as f64);
            // Exact decay a(t) = exp(−2 ν t), at R.
            let a_exact = (-(lift(2.0) * lift(nu) * t_final)).exp();
            report.add_series("amplitude_final", vec![a_final]);
            report.add_series("amplitude_exact", vec![a_exact]);
        }

        Ok(report)
    }
}

/// The largest component-wise absolute difference between two vectors.
fn max_abs_diff<R: CfdScalar>(a: &[R; 3], b: &[R; 3]) -> R {
    (0..3)
        .map(|i| (a[i] - b[i]).abs())
        .fold(R::zero(), |acc, x| if acc > x { acc } else { x })
}

fn scale_3<R: CfdScalar>(v: [R; 3], s: R) -> [R; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn scale_3x3<R: CfdScalar>(m: [[R; 3]; 3], s: R) -> [[R; 3]; 3] {
    core::array::from_fn(|i| core::array::from_fn(|j| m[i][j] * s))
}
