/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # QTT Sod shock tube — the compressible-flux gate (Tier-B Stage 2)
//!
//! Marches the 1-D conservative compressible Euler equations in quantized-tensor-train form
//! (`CompressibleEuler1d`: ideal gas + Rusanov flux, conservative divergence via the §0 MPOs) on the
//! Sod shock-tube initial data, and self-verifies (exit nonzero on break) against the **exact Riemann
//! solution**: density / velocity / pressure profiles, with the left expansion fan, the contact, and
//! the right shock at the correct speeds.
//!
//! The periodic operators are run on a wide domain `[-1, 1]` so the boundary-jump waves stay outside
//! the measurement window `|x| ≤ 0.5` at the final time.
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example qtt_sod
//! ```

mod exact_riemann;
mod print_utils;

use deep_causality_cfd::CompressibleEuler1d;
use deep_causality_tensor::Truncation;
use exact_riemann::Prim;

/// `2^L` cells over the domain.
const L: usize = 9;
const GAMMA: f64 = 1.4;
const CFL: f64 = 0.4;
const T_FINAL: f64 = 0.2;
const X0: f64 = -1.0;
const X1: f64 = 1.0;

fn main() {
    println!("=== QTT Sod shock tube: compressible Euler (Rusanov, tensor-train) ===\n");
    let n = 1usize << L;
    let dx = (X1 - X0) / n as f64;
    println!(
        "Case: gamma = {GAMMA}, CFL = {CFL}, t = {T_FINAL}, domain [{X0}, {X1}], {n} cells, precision f64\n"
    );

    let trunc = Truncation::<f64>::by_tol(1e-8).unwrap_or_else(|e| {
        eprintln!("truncation: {e:?}");
        std::process::exit(2);
    });
    let solver = CompressibleEuler1d::<f64>::new(L, dx, GAMMA, CFL, trunc)
        .unwrap_or_else(|e| fail("Sod solver assembly", e));

    let left = Prim {
        rho: 1.0,
        u: 0.0,
        p: 1.0,
    };
    let right = Prim {
        rho: 0.125,
        u: 0.0,
        p: 0.1,
    };
    let xc = |i: usize| X0 + (i as f64 + 0.5) * dx;

    // Conservative Sod initial data (discontinuity at x = 0).
    let mut rho0 = vec![0.0f64; n];
    let mut mom0 = vec![0.0f64; n];
    let mut e0 = vec![0.0f64; n];
    for i in 0..n {
        let s = if xc(i) < 0.0 { left } else { right };
        rho0[i] = s.rho;
        mom0[i] = s.rho * s.u;
        e0[i] = s.p / (GAMMA - 1.0) + 0.5 * s.rho * s.u * s.u;
    }

    let (rho, mom, energy) = solver
        .run(&(rho0, mom0, e0), T_FINAL)
        .unwrap_or_else(|e| fail("Sod march", e));

    print_utils::render(n, dx, X0, T_FINAL, GAMMA, left, right, &rho, &mom, &energy);
    let errs = print_utils::errors(n, dx, X0, T_FINAL, GAMMA, left, right, &rho, &mom, &energy);
    if print_utils::verify(&errs) {
        print_utils::summary();
    } else {
        std::process::exit(1);
    }
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
