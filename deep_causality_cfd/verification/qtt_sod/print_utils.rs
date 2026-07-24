/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Measure + verify for the Sod gate: the L1 error of the QTT marcher against the exact Riemann
//! solution over the boundary-clean window `|x| ≤ 0.5`.

use crate::exact_riemann::{Prim, sample};
use deep_causality_cfd::{EvidenceClass, ideal_gas_pressure};

/// Mean-absolute (L1) errors of the marched primitives vs the exact Riemann solution.
pub struct Errors {
    pub rho: f64,
    pub u: f64,
    pub p: f64,
}

/// The measurement window half-width (away from the periodic boundary waves).
const WINDOW: f64 = 0.5;
/// L1 tolerance — first-order Rusanov smears the contact, so the bound reflects mean accuracy.
///
/// Evidence class: **reference**. The comparison is against the exact Riemann solution of the Sod
/// problem (Sod 1978; Toro, *Riemann Solvers and Numerical Methods for Fluid Dynamics*, ch. 4),
/// computed independently of the solver, so clearing this bound is evidence about the physics —
/// not merely evidence of non-regression. The tolerance itself is sized by the first-order scheme's
/// known smearing of the contact, not pinned from a measured run.
const TOL: f64 = 0.03;
/// Evidence class of the Sod gates. See [`TOL`].
const SOD_EVIDENCE: EvidenceClass = EvidenceClass::Reference;

#[allow(clippy::too_many_arguments)]
pub fn errors(
    n: usize,
    dx: f64,
    x0: f64,
    t: f64,
    gamma: f64,
    left: Prim,
    right: Prim,
    rho: &[f64],
    mom: &[f64],
    energy: &[f64],
) -> Errors {
    let (mut er, mut eu, mut ep, mut cnt) = (0.0f64, 0.0f64, 0.0f64, 0usize);
    for i in 0..n {
        let x = x0 + (i as f64 + 0.5) * dx;
        if x.abs() > WINDOW {
            continue;
        }
        let un = mom[i] / rho[i];
        let pn = ideal_gas_pressure(rho[i], mom[i], energy[i], gamma);
        let ex = sample(left, right, gamma, x / t);
        er += (rho[i] - ex.rho).abs();
        eu += (un - ex.u).abs();
        ep += (pn - ex.p).abs();
        cnt += 1;
    }
    let c = cnt as f64;
    Errors {
        rho: er / c,
        u: eu / c,
        p: ep / c,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render(
    n: usize,
    dx: f64,
    x0: f64,
    t: f64,
    gamma: f64,
    left: Prim,
    right: Prim,
    rho: &[f64],
    mom: &[f64],
    energy: &[f64],
) {
    println!("Profiles (numeric vs exact) at a few stations:");
    println!(
        "  {:>7} | {:>18} | {:>18}",
        "x", "rho (num / exact)", "p (num / exact)"
    );
    for &x in &[-0.4f64, -0.1, 0.1, 0.3, 0.45] {
        let i = (((x - x0) / dx - 0.5).round() as i64).clamp(0, n as i64 - 1) as usize;
        let pn = ideal_gas_pressure(rho[i], mom[i], energy[i], gamma);
        let ex = sample(left, right, gamma, x / t);
        println!(
            "  {:>7.3} | {:>8.4} / {:>7.4} | {:>8.4} / {:>7.4}",
            x, rho[i], ex.rho, pn, ex.p
        );
    }
}

pub fn verify(e: &Errors) -> bool {
    println!(
        "\n--- Sod gate (L1 error over |x| <= {WINDOW}, tol {TOL}; reference: exact Riemann solution) ---"
    );
    let ok = |label: &str, v: f64| {
        let pass = v < TOL;
        println!(
            "  [{}] [{SOD_EVIDENCE}] {label} L1 error = {v:.4}",
            if pass { "PASS" } else { "FAIL" }
        );
        pass
    };
    let a = ok("density ", e.rho);
    let b = ok("velocity", e.u);
    let c = ok("pressure", e.p);
    a && b && c
}

pub fn summary() {
    println!(
        "\n=== Sod profiles match the exact Riemann solution — compressible flux verified. ==="
    );
}
