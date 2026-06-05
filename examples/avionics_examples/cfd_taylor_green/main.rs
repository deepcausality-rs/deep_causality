/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Avionics CFD verification by the Method of Manufactured Solutions
//!
//! The Taylor-Green vortex is a closed-form solution of the incompressible Navier-Stokes
//! equations, so a correct right-hand-side kernel, fed the exact spatial derivatives, must return
//! the exact time derivative; a correct time march must then track the exact decay.
//!
//! Three DeepCausality abstractions appear together here:
//!
//! - **The tangent functor** makes the manufactured derivatives exact and free of finite
//!   differences. `gradient` yields the velocity Jacobian `∇u` and the pressure gradient `∇p`;
//!   nested duals yield the Laplacian `∇²u`.
//! - **The integration operator** (`Rk4`) marches the amplitude with the kernel in the loop.
//! - **The causal monad** sequences the two verification stages. Each binds to the next, and a
//!   kernel failure short-circuits the chain through the effect's error channel.
//!
//! Precision is a parameter: change the `FloatType` alias to re-run the whole computation at a
//! different precision, which matters for fluid dynamics.
//!
//! Taylor-Green vortex, 2-D embedded in 3-D with `w = 0`:
//!
//!     u =  sin x · cos y · F(t),   v = −cos x · sin y · F(t),   w = 0,
//!     p = (ρ/4)(cos 2x + cos 2y) · F(t)²,   F(t) = exp(−2 ν t).

mod model;
mod print_utils;

use deep_causality_calculus::{EndoArrow, Rk4};
use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum, PropagatingEffect};

/// The working precision for the whole CFD computation. **This is the single alias to change**:
/// the autodiff scalar, the Navier-Stokes kernel arithmetic, and the Rk4 march all run at this
/// precision. Switching to `f32` or `Float106` (the latter also needs
/// `use deep_causality_num::Float106;`) re-runs everything at that precision. The stage-1 residual
/// then tracks machine epsilon: ~3e-8 at f32, ~1e-16 at f64, ~8e-33 at Float106.
pub type FloatType = f64;

fn main() {
    println!("=== Avionics CFD verification: Taylor-Green via Manufactured Solutions ===\n");

    // Physical constants are exact `f64` specifications. They enter the computation only through
    // `from_f64`, which lifts them losslessly into the working precision `FloatType`; every computed
    // quantity below (the field, its derivatives, the kernel, the Rk4 march) runs at `FloatType`.
    let nu = 0.05_f64; // kinematic viscosity (m²/s)
    let rho = 1.0_f64; // density (kg/m³)
    let base: [FloatType; 3] = [model::ft(0.7), model::ft(1.1), model::ft(0.0)];
    let t0 = 0.0_f64; // sample time

    // The verification runs as a monadic chain: differentiate-then-kernel, then march. The second
    // stage binds onto the first, so a kernel error in stage 1 short-circuits before the march.
    CausalFlow::effect()
        .and_then(move |_| stage1(base, nu, rho, t0).into())
        .and_then(move |s1| stage2(s1, nu, rho).into())
        .run(
            |report| print_utils::print_report(&report),
            |err| eprintln!("CFD pipeline failed: {err:?}"),
        );
}

/// Stage 1: exact spatial derivatives feed the Navier-Stokes kernel; compare to the exact
/// `∂u/∂t = −2 ν u`. A kernel failure returns an error effect that halts the chain.
fn stage1(base: [FloatType; 3], nu: f64, rho: f64, t0: f64) -> PropagatingEffect<Stage1> {
    let u = model::velocity(base, nu, t0);
    let grad_u = model::velocity_jacobian(base, nu, t0);
    let lap_u = model::velocity_laplacian(base, nu, t0);
    let grad_p = model::pressure_gradient(base, nu, t0, rho);

    match model::navier_stokes_rhs(&u, &grad_u, &lap_u, &grad_p, nu, rho) {
        Ok(dudt) => {
            let exact_dudt = model::exact_time_derivative(&u, nu);
            let kernel_err = model::max_abs_diff(&dudt, &exact_dudt);
            PropagatingEffect::pure(Stage1 {
                u,
                grad_u,
                lap_u,
                grad_p,
                dudt,
                exact_dudt,
                kernel_err,
            })
        }
        Err(e) => error_effect(&format!("Navier-Stokes kernel: {e:?}")),
    }
}

/// Stage 2: `Rk4` marches the amplitude with the kernel in the loop; compare to `exp(−2 ν t)`.
fn stage2(s1: Stage1, nu: f64, rho: f64) -> PropagatingEffect<Report> {
    let rate = model::amplitude_rate(s1.u, s1.grad_u, s1.lap_u, s1.grad_p, nu, rho);
    let dt = model::ft(0.005);
    let steps = 200usize;
    let t_final = dt * model::ft(steps as f64);
    let a_final = Rk4::new(dt, rate).iterate_n(model::ft(1.0), steps);
    let a_exact = model::exact_decay(nu, t_final);
    PropagatingEffect::pure(Report {
        s1,
        a_final,
        a_exact,
        t_final,
        steps,
    })
}

fn error_effect<T: Default + Clone + std::fmt::Debug>(msg: &str) -> PropagatingEffect<T> {
    PropagatingEffect::from_error(CausalityError::new(CausalityErrorEnum::Custom(msg.into())))
}

/// Output of stage 1: the field at the sample point, its exact spatial derivatives, the kernel
/// time derivative, and the residual against the manufactured solution.
#[derive(Default, Clone, Debug)]
struct Stage1 {
    u: [FloatType; 3],
    grad_u: [[FloatType; 3]; 3],
    lap_u: [FloatType; 3],
    grad_p: [FloatType; 3],
    dudt: [FloatType; 3],
    exact_dudt: [FloatType; 3],
    kernel_err: FloatType,
}

/// Output of stage 2: the marched amplitude against the exact decay.
#[derive(Default, Clone, Debug)]
struct Report {
    s1: Stage1,
    a_final: FloatType,
    a_exact: FloatType,
    t_final: FloatType,
    steps: usize,
}
