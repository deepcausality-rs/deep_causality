/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Avionics CFD verification by the Method of Manufactured Solutions — via the CfdFlow DSL
//!
//! The Taylor-Green vortex is a closed-form solution of the incompressible Navier-Stokes
//! equations, so a correct right-hand-side kernel, fed the exact spatial derivatives, must return
//! the exact time derivative; a correct time march must then track the exact decay.
//!
//! The manufactured solution and the verification workflow now live in the `deep_causality_cfd`
//! corpus: [`TaylorGreen`](deep_causality_cfd::TaylorGreen) supplies the exact derivatives through
//! the **tangent functor** (autodiff — no finite differences), and `CfdFlow::verify` runs the
//! kernel residual plus the kernel-in-the-loop `Rk4` amplitude march. The example just declares the
//! config and renders the report. A genuinely novel manufactured field would instead implement the
//! [`Manufactured`](deep_causality_cfd::Manufactured) seam in place.

mod config;
mod print_utils;

use deep_causality_cfd::CfdFlow;

/// The working precision for the whole CFD computation. **This is the single alias to change**:
/// the autodiff scalar, the Navier-Stokes kernel arithmetic, and the `Rk4` march all run at this
/// precision (`f32`, `f64`, or `Float106` with `use deep_causality_num::Float106;`). The stage-1
/// residual tracks machine epsilon: ~3e-8 at f32, ~1e-16 at f64, ~8e-33 at Float106.
pub type FloatType = f64;

fn main() {
    println!("=== Avionics CFD verification: Taylor-Green via Manufactured Solutions ===\n");

    // Configuration (the manufactured solution + sample point + amplitude march) is declared in
    // `config`; the CfdFlow DSL runs the verification workflow; `print_utils` renders the report.
    let config = config::build_verify_config().unwrap_or_else(|e| fail("CFD MMS configuration", e));
    let report = CfdFlow::verify(&config)
        .run()
        .unwrap_or_else(|e| fail("CFD MMS pipeline", e));

    print_utils::render(&report);
    if !print_utils::verify(&report) {
        std::process::exit(1);
    }
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
