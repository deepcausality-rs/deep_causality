// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! QTT acoustic-implicit study — does the split-operator preconditioner de-risk the implicit step?
//! (Resolution 6 / design D10, the second Tier-B make-or-break.)
//!
//! The implicit-acoustic operator is `A = I − Δt²·c(x)²·∂²` (backward-Euler on the fast pressure mode;
//! SPD because `−∂²` is positive). Resolution 6 splits it `A = A₀ + A₁` with `A₀ = I − Δt²·c̄²·∂²` the
//! **constant-coefficient** core (a known low-rank inverse) and `A₁ = −Δt²·(c²(x) − c̄²)·∂²` the
//! variable remainder, and claims: (a) `A₀⁻¹` is **low-rank and resolution-stable**, and (b) on a
//! **smooth** sound-speed field `‖A₀⁻¹A₁‖ < 1`, so the preconditioned solve `A₀⁻¹A = I + A₀⁻¹A₁`
//! contracts — converting "does AMEn converge?" into a measurable perturbation bound. The honest
//! counterpart: across a **captured shock** (a sharp `c` jump) the bound degrades — which is *why*
//! shock-fitting (Res 5), by keeping the interior smooth, is what makes the implicit step cheap.
//!
//! Measurements:
//!   AC-A  the constant-coefficient solve `A₀⁻¹·b` has bounded bond, resolution-stable (L=8 vs L=10).
//!   AC-B  smooth interior: `ρ(A₀⁻¹A₁) < 1` by a comfortable margin (power iteration).
//!   AC-C  captured jump: `ρ` degrades sharply toward / past 1 — the jump is the hard part, not the bulk.
//!
//! Self-verifying: gates encode the finding and exit non-zero on regression.

use deep_causality_cfd::{laplacian, quantize};
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, SolveConfig, TensorTrain,
    TensorTrainOperator, Truncation, solve,
};

const PI: f64 = std::f64::consts::PI;
/// Implicit-acoustic stiffness `s = Δt²·c̄²/Δx²` (> 1 ⇒ the acoustic CFL the IMEX step removes).
const STIFF: f64 = 8.0;
const MAX_RANK: usize = 32;
const SOLVE_TOL: f64 = 1e-6;
const SOLVE_SWEEPS: usize = 300;

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let tol = 1e-9;

    println!("=== QTT acoustic-implicit preconditioner (Res 6 / D10), stiffness s = {STIFF} ===\n");

    // ---- AC-A: the constant-coefficient inverse is low-rank and resolution-stable. ----
    let (bond8, res8) = const_inverse_bond(8, tol);
    let (bond10, res10) = const_inverse_bond(10, tol);
    println!(
        "  AC-A  A0^-1·b (smooth RHS): bond  L8={bond8} (res {res8:.1e})   L10={bond10} (res {res10:.1e})"
    );
    if bond8 > 16 || bond10 > 16 || bond10 > bond8 + 4 {
        failures.push(format!(
            "AC-A: constant-coeff inverse not low-rank / resolution-stable (L8={bond8}, L10={bond10})"
        ));
    }

    // ---- AC-B / AC-C: the perturbation spectral radius, smooth interior vs captured jump. ----
    let l = 7usize;
    let rho_smooth = spectral_radius(l, Coeff::Smooth);
    let rho_jump = spectral_radius(l, Coeff::Jump);
    println!("\n  AC-B  smooth interior  : rho(A0^-1 A1) = {rho_smooth:.3}");
    println!("  AC-C  captured c-jump  : rho(A0^-1 A1) = {rho_jump:.3}");

    // Gate AC-B: a smooth sound-speed field gives a contracting preconditioner with margin.
    if rho_smooth >= 0.8 {
        failures.push(format!(
            "AC-B: smooth-interior preconditioner did not contract (rho={rho_smooth:.3} >= 0.8)"
        ));
    }
    // Gate AC-C: a captured jump degrades the bound toward the divergence threshold (rho -> 1) and far
    // past the smooth case. The jump, not the bulk, is the hard part — so fitting, which removes the
    // interior jump, is what keeps the implicit step cheap.
    if rho_jump < 0.82 || rho_jump < 1.35 * rho_smooth {
        failures.push(format!(
            "AC-C: captured jump did not degrade the preconditioner toward 1 (jump={rho_jump:.3}, smooth={rho_smooth:.3})"
        ));
    }

    println!("\n--- reading ---");
    println!(
        "  The constant-coefficient core inverts at bounded bond, resolution-stable (L8={bond8}, L10={bond10}),"
    );
    println!("  so A0^-1 is a cheap preconditioner — no AMEn-convergence gamble on the core.");
    println!(
        "  On a SMOOTH interior rho={rho_smooth:.2} < 1: the preconditioned operator I + A0^-1 A1 contracts,"
    );
    println!("  i.e. the implicit step converges geometrically (the Res-6 claim, measured).");
    println!(
        "  Across a CAPTURED jump rho={rho_jump:.2} ({:.1}x worse): the jump is the hard part — which is",
        rho_jump / rho_smooth
    );
    println!(
        "  exactly why shock-FITTING (Res 5) keeps the interior smooth so the solve stays cheap."
    );

    if failures.is_empty() {
        println!(
            "\nALL GATES PASSED — the split + closed-form-core preconditioner de-risks the implicit acoustic step."
        );
    } else {
        eprintln!("\nFAILED GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

/// Sound-speed coefficient family.
#[derive(Clone, Copy)]
enum Coeff {
    /// Mild smooth variation (a fitted interior).
    Smooth,
    /// A sharp sound-speed jump (×5) at mid-domain (a captured shock).
    Jump,
}

/// `c(x)²` sampled on the `2^l` grid for a coefficient family.
fn c2_profile(l: usize, coeff: Coeff) -> Vec<f64> {
    let n = 1usize << l;
    (0..n)
        .map(|i| {
            let x = i as f64 / n as f64;
            match coeff {
                Coeff::Smooth => {
                    let c = 1.0 + 0.3 * (2.0 * PI * x).sin();
                    c * c
                }
                Coeff::Jump => {
                    // c² jumps 1 → 25 (sound speed ×5, a strong post-shock contrast).
                    if x < 0.5 { 1.0 } else { 25.0 }
                }
            }
        })
        .collect()
}

/// L2 norm of a tensor train (dense, cheap at study sizes).
fn l2(t: &CausalTensorTrain<f64>) -> f64 {
    t.to_dense()
        .expect("to_dense")
        .as_slice()
        .iter()
        .map(|v| v * v)
        .sum::<f64>()
        .sqrt()
}

/// Build the constant-coefficient implicit operator `A₀ = I − β·∂²` with `β = s·Δx²` (stiffness `s`).
fn build_a0(l: usize, trunc: &Truncation<f64>) -> CausalTensorTrainOperator<f64> {
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let beta = STIFF * dx * dx;
    let lap = laplacian::<f64>(l, dx, trunc).expect("lap");
    let id = CausalTensorTrainOperator::<f64>::identity(&vec![2usize; l]);
    id.add(&lap.scale(-beta)).expect("A0 = I - beta*lap")
}

/// AC-A: solve `A₀ x = b` for a smooth `b` and report `(x.max_bond, relative residual)`.
fn const_inverse_bond(l: usize, tol: f64) -> (usize, f64) {
    let trunc = Truncation::<f64>::by_tol(tol).expect("tol");
    let n = 1usize << l;
    let a0 = build_a0(l, &trunc);
    let b_dense: Vec<f64> = (0..n)
        .map(|i| {
            let x = i as f64 / n as f64;
            (2.0 * PI * x).sin() + 0.5 * (4.0 * PI * x).cos()
        })
        .collect();
    let b = quantize(&CausalTensor::new(b_dense, vec![n]).expect("dense"), &trunc).expect("b");
    let cfg = SolveConfig::<f64>::new(SOLVE_SWEEPS, SOLVE_TOL, 1e-13).expect("cfg");
    let x = solve::linear(&a0, &b, MAX_RANK, &cfg).expect("A0 solve");
    let ax = a0.apply(&x, &trunc).expect("A0 x");
    let res = l2(&ax.add(&b.scale(-1.0)).expect("residual")) / (l2(&b) + 1e-300);
    (x.max_bond(), res)
}

/// AC-B/AC-C: spectral radius of `M = A₀⁻¹A₁`, where `A₁ = −Δt²·diag(c²(x) − c̄²)·∂²` and
/// `Δt² = β / c̄²`. The spectral radius is a coordinate-free fact about the operator pencil, so it is
/// computed densely at this small size (an `n × n` matrix), free of any iterative-solver tolerance.
fn spectral_radius(l: usize, coeff: Coeff) -> f64 {
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let beta = STIFF * dx * dx;
    let lap = dense_periodic_laplacian(n, dx);

    let c2 = c2_profile(l, coeff);
    let cbar2 = c2.iter().sum::<f64>() / n as f64;
    let dt2 = beta / cbar2;

    // A0 = I − β·L ;  A1 = −Δt²·diag(c²−c̄²)·L  (row i scaled by c²(x_i) − c̄²).
    let mut a0 = vec![0.0f64; n * n];
    let mut a1 = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..n {
            let lij = lap[i * n + j];
            a0[i * n + j] = if i == j { 1.0 } else { 0.0 } - beta * lij;
            a1[i * n + j] = -dt2 * (c2[i] - cbar2) * lij;
        }
    }
    let a0_inv = dense_invert(&a0, n);
    let m = dense_matmul(&a0_inv, &a1, n);

    // Power iteration on M for its spectral radius.
    let mut v: Vec<f64> = (0..n)
        .map(|i| {
            let x = i as f64 / n as f64;
            (2.0 * PI * x).sin() + 0.4 * (4.0 * PI * x).sin() + 0.2
        })
        .collect();
    let mut ratio = 0.0;
    for _ in 0..200 {
        let mv = dense_matvec(&m, &v, n);
        let nv = vnorm(&mv);
        ratio = nv / (vnorm(&v) + 1e-300);
        let inv = 1.0 / (nv + 1e-300);
        v = mv.iter().map(|x| x * inv).collect();
    }
    ratio
}

/// Periodic central second-difference matrix `∂²` (row-major `n × n`).
fn dense_periodic_laplacian(n: usize, dx: f64) -> Vec<f64> {
    let inv = 1.0 / (dx * dx);
    let mut l = vec![0.0f64; n * n];
    for i in 0..n {
        l[i * n + i] = -2.0 * inv;
        l[i * n + (i + 1) % n] += inv;
        l[i * n + (i + n - 1) % n] += inv;
    }
    l
}

/// Inverse of a dense `n × n` matrix by Gauss–Jordan with partial pivoting.
fn dense_invert(a: &[f64], n: usize) -> Vec<f64> {
    let mut m = a.to_vec();
    let mut inv = vec![0.0f64; n * n];
    for i in 0..n {
        inv[i * n + i] = 1.0;
    }
    for col in 0..n {
        let mut piv = col;
        let mut best = m[col * n + col].abs();
        for r in (col + 1)..n {
            let v = m[r * n + col].abs();
            if v > best {
                best = v;
                piv = r;
            }
        }
        if piv != col {
            for j in 0..n {
                m.swap(col * n + j, piv * n + j);
                inv.swap(col * n + j, piv * n + j);
            }
        }
        let d = m[col * n + col];
        for j in 0..n {
            m[col * n + j] /= d;
            inv[col * n + j] /= d;
        }
        for r in 0..n {
            if r == col {
                continue;
            }
            let f = m[r * n + col];
            if f != 0.0 {
                for j in 0..n {
                    m[r * n + j] -= f * m[col * n + j];
                    inv[r * n + j] -= f * inv[col * n + j];
                }
            }
        }
    }
    inv
}

/// Dense `n × n` matrix product `a·b` (row-major).
fn dense_matmul(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let mut c = vec![0.0f64; n * n];
    for i in 0..n {
        for k in 0..n {
            let aik = a[i * n + k];
            if aik != 0.0 {
                for j in 0..n {
                    c[i * n + j] += aik * b[k * n + j];
                }
            }
        }
    }
    c
}

/// Dense matrix–vector product `a·v`.
fn dense_matvec(a: &[f64], v: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; n];
    for i in 0..n {
        let mut s = 0.0;
        for j in 0..n {
            s += a[i * n + j] * v[j];
        }
        out[i] = s;
    }
    out
}

/// Euclidean norm of a dense vector.
fn vnorm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}
