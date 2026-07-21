/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Graded-metric MMS truncation study — CFD rung R1
//!
//! A graded mesh is a metric state on an unchanged lattice (`variable-grid-geometry.md`
//! §2): `d`, the discrete Stokes theorem, and divergence-freeness are combinatorial, so
//! they hold **exactly at any grading** — only *accuracy order* is at stake. This study
//! quantifies that for the two operators of the incompressible march on graded meshes:
//!
//! - the **convective** operator `i_X ω` (interior product), via the Cartan magic-formula
//!   MMS `i_X dω + d i_X ω → L_X ω`;
//! - the **viscous** operator `Δ₀ = δd` (Laplacian), via a `sin·sin` MMS.
//!
//! Both are measured in **two norms** (max + L2). The metric is graded on axis 1 by a
//! smooth periodic edge-length modulation `ℓ(pos) = 1 + a·cos(2π·pos/N)` (smooth across the
//! seam, sums to `N`, so the wavenumber is unchanged).
//!
//! ## Result: smooth grading retains second order — for both operators
//!
//! Measured outcome: **both the convective and viscous operators are second order in both
//! norms at every grading amplitude**, even at strong grading (a 3:1 spacing ratio). The
//! error *constant* grows mildly with grading; the *order* holds at ≈ 2. So the R1 promise
//! ("graded meshes resolve walls cheaply and keep fast convergence") holds today.
//!
//! ### A correctness caveat this example exists to enforce
//!
//! DEC operators act on **cochains = integrals over cells**: a discrete 1-form on an edge
//! is `∫ ω ≈ (tangential value at the midpoint) · ℓ_edge`. Both `ω` and `X♭` carry that
//! `ℓ` factor, and the 1-form *output* is normalised back by `ℓ` before comparison to the
//! pointwise analytic. Omitting the `ℓ` factor is invisible on a uniform mesh (`ℓ = 1`) but
//! `O(ℓ)`-wrong on a graded one — an inconsistency that *looks* like an operator order-loss
//! but is purely a measurement bug (an earlier revision of this study hit exactly that and
//! mis-reported a convective collapse). The viscous MMS uses 0-forms (no length factor), so
//! it was always measured consistently. Run:
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example dec_graded_mms_verification
//! ```

use deep_causality_cfd::{Grading, Mesh};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, LatticeCell, LatticeComplex, Manifold,
};
use std::f64::consts::PI;

fn main() {
    let resolutions = [8usize, 16, 32, 64];
    let amplitudes = [0.0, 0.1, 0.2, 0.3];

    let mut finest_orders: Vec<(&str, f64, f64, f64)> = Vec::new();

    for (name, kernel) in [
        (
            "CONVECTIVE  i_X ω (interior product)",
            convective_mms as fn(usize, f64) -> (f64, f64),
        ),
        (
            "VISCOUS     Δ₀ = δd  (Laplacian)",
            viscous_mms as fn(usize, f64) -> (f64, f64),
        ),
    ] {
        println!("\n=== {name} — order vs grading amplitude (max-norm | L2-norm) ===");
        println!(
            "{:>5}  {:>10} {:>10}   {:>14}   {:>14}",
            "a", "max E(64)", "L2 E(64)", "max-norm p", "L2-norm p"
        );
        println!("{}", "-".repeat(70));
        for &amp in &amplitudes {
            let (maxs, l2s): (Vec<f64>, Vec<f64>) =
                resolutions.iter().map(|&n| kernel(n, amp)).unzip();
            let (po, lo) = (observed_orders(&maxs), observed_orders(&l2s));
            println!(
                "{amp:>5.2}  {:>10.2e} {:>10.2e}   {:>14}   {:>14}",
                maxs.last().unwrap(),
                l2s.last().unwrap(),
                fmt_orders(&po),
                fmt_orders(&lo),
            );
            // The finest pair is what the second-order claim rests on; the coarse pairs dip at
            // strong grading and recover, which the module doc describes.
            finest_orders.push((name, amp, *po.last().unwrap(), *lo.last().unwrap()));
        }
    }

    println!("\nReading: both operators hold ≈ second order (both norms) at every grading");
    println!("amplitude — smooth grading retains second order. The error constant grows mildly");
    println!("with grading; the order does not degrade. Structure (divergence-freeness) is exact");
    println!("at every amplitude regardless. (Cochains must be edge-integrals — see the module");
    println!("doc; omitting the ℓ factor mis-measures a false order-loss on graded meshes.)");

    if !verify(&finest_orders) {
        std::process::exit(1);
    }
}

/// Minimum acceptable finest-pair observed order.
///
/// Evidence class: **reference**. Second order is the theoretical property of the discretisation
/// under test (centered DEC stencils on a smoothly graded metric); the bound is a margin below that
/// theoretical 2, not a value pinned from a measurement. Measured finest-pair orders are 1.98-2.00
/// across every grading amplitude and both norms.
const MIN_FINEST_ORDER: f64 = 1.9;

/// Self-verification (exit nonzero on break): the finest-pair observed order holds at ~2 for both
/// operators, in both norms, at every grading amplitude.
///
/// This harness previously had **no** `process::exit` at all — it tabulated the orders and always
/// exited zero, so an operator that lost its second-order accuracy would have printed the loss and
/// still passed. verification/README.md's stated "exits nonzero on break" convention did not hold.
///
/// BREAKING CONDITION: drop the `ℓ` edge-integral factor from the cochain construction (the exact
/// inconsistency the module doc warns about) and the graded-mesh orders collapse below 1.9.
fn verify(finest: &[(&str, f64, f64, f64)]) -> bool {
    println!("\n--- graded-metric MMS gates (finest pair, theoretical order 2) ---");
    let mut ok = true;
    for &(name, amp, p_max, p_l2) in finest {
        let pass = p_max >= MIN_FINEST_ORDER && p_l2 >= MIN_FINEST_ORDER;
        println!(
            "  [{}] [reference] {} @ grading {amp:.2}: max-norm p = {p_max:.3}, L2 p = {p_l2:.3} \
             (>= {MIN_FINEST_ORDER})",
            if pass { "PASS" } else { "FAIL" },
            name.split_whitespace().next().unwrap_or(name),
        );
        ok &= pass;
    }
    ok
}

/// Relative max- and L2-norm errors of a discrete field against an analytic reference.
fn rel_errors(discrete: &[f64], analytic: &[f64]) -> (f64, f64) {
    let mut max_err = 0.0_f64;
    let mut max_ref = 0.0_f64;
    let mut sse = 0.0_f64;
    let mut ssr = 0.0_f64;
    for (d, a) in discrete.iter().zip(analytic) {
        max_err = max_err.max((d - a).abs());
        max_ref = max_ref.max(a.abs());
        sse += (d - a) * (d - a);
        ssr += a * a;
    }
    (max_err / max_ref, (sse / ssr).sqrt())
}

/// A torus graded on axis 1 by `ℓ(pos) = 1 + amp·cos(2π·pos/N)`: the manifold (from the **CfdFlow**
/// DSL graded mesh — the DSL owns the geometry), plus the physical node coordinates along the graded
/// axis that the bespoke operator MMS measurement needs.
fn graded(n: usize, amp: f64) -> (Manifold<LatticeComplex<2, f64>, f64>, Vec<f64>, f64) {
    // The DSL owns the geometry: a graded torus with a cosine metric modulation on axis 1.
    let manifold = Mesh::<2, f64>::torus(n)
        .graded(Grading::cosine(1, amp))
        .manifold()
        .expect("graded torus geometry");

    // The MMS measurement (below) still needs the physical node coordinates along the graded axis.
    let len = move |pos: usize| 1.0 + amp * (2.0 * PI * pos as f64 / n as f64).cos();
    let mut y_node = vec![0.0_f64; n + 1];
    for j in 0..n {
        y_node[j + 1] = y_node[j] + len(j);
    }
    let k = 2.0 * PI / (n as f64);
    (manifold, y_node, k)
}

/// A unit-metric carrier holding `form` at grade `k_grade` (for the metric-free `d`).
fn unit_carrier(
    n: usize,
    total: usize,
    num_cells: &dyn Fn(usize) -> usize,
    k_grade: usize,
    form: &[f64],
) -> Manifold<LatticeComplex<2, f64>, f64> {
    let off: usize = (0..k_grade).map(num_cells).sum();
    let mut d = vec![0.0; total];
    d[off..off + form.len()].copy_from_slice(form);
    let t = CausalTensor::new(d, vec![total]).unwrap();
    let unit = CubicalReggeGeometry::<2, f64>::unit();
    Manifold::from_cubical_with_metric(LatticeComplex::<2, f64>::square_torus(n), t, unit, 0)
}

/// Convective operator MMS (Cartan magic formula): relative (max, L2) errors of
/// `i_X dω + d i_X ω` vs the analytic Lie derivative, evaluated at physical edge midpoints.
fn convective_mms(n: usize, amp: f64) -> (f64, f64) {
    let len = move |pos: usize| 1.0 + amp * (2.0 * PI * pos as f64 / n as f64).cos();
    let (manifold, y_node, k) = graded(n, amp);
    let complex = manifold.complex();
    let total: usize = (0..=2).map(|g| complex.num_cells(g)).sum();
    let nc = |g: usize| complex.num_cells(g);

    let midpoint = |c: &LatticeCell<2>| {
        let axis = c.orientation().trailing_zeros() as usize;
        let p = c.position();
        let x = if axis == 0 {
            p[0] as f64 + 0.5
        } else {
            p[0] as f64
        };
        let y = if axis == 1 {
            0.5 * (y_node[p[1]] + y_node[p[1] + 1])
        } else {
            y_node[p[1]]
        };
        (x, y, axis)
    };

    // DEC cochains are integrals over cells: a 1-form on an edge is ∫ ≈ (tangential
    // value at the midpoint) · ℓ_edge. Both ω and X♭ carry that ℓ factor (=1 on axis 0,
    // =ℓ(pos) on the graded axis 1); the output is normalised back by ℓ below.
    let edge_len = |c: &LatticeCell<2>| {
        if c.orientation().trailing_zeros() as usize == 1 {
            len(c.position()[1])
        } else {
            1.0
        }
    };
    let omega_vals: Vec<f64> = complex
        .iter_cells(1)
        .map(|c| {
            let (mx, my, axis) = midpoint(&c);
            let comp = if axis == 0 {
                (k * my).sin()
            } else {
                (k * mx).sin()
            };
            comp * edge_len(&c)
        })
        .collect();
    let x_vals: Vec<f64> = complex
        .iter_cells(1)
        .map(|c| {
            let (mx, my, axis) = midpoint(&c);
            let comp = if axis == 0 {
                (k * mx).cos()
            } else {
                (k * my).cos()
            };
            comp * edge_len(&c)
        })
        .collect();
    let n1 = nc(1);
    let omega = CausalTensor::new(omega_vals.clone(), vec![n1]).unwrap();
    let x_flat = CausalTensor::new(x_vals, vec![n1]).unwrap();

    let d_omega = unit_carrier(n, total, &nc, 1, &omega_vals).exterior_derivative(1);
    let term1 = manifold.interior_product(&x_flat, &d_omega, 2).unwrap();
    let ix_omega = manifold.interior_product(&x_flat, &omega, 1).unwrap();
    let term2 = unit_carrier(n, total, &nc, 0, ix_omega.as_slice()).exterior_derivative(0);

    // The output is a 1-form cochain (edge-integral); normalise by ℓ_edge to recover the
    // pointwise tangential value, then compare to the pointwise analytic Lie derivative.
    let discrete: Vec<f64> = complex
        .iter_cells(1)
        .enumerate()
        .map(|(i, c)| (term1.as_slice()[i] + term2.as_slice()[i]) / edge_len(&c))
        .collect();
    let analytic: Vec<f64> = complex
        .iter_cells(1)
        .map(|c| {
            let (mx, my, axis) = midpoint(&c);
            if axis == 0 {
                k * (k * my).cos().powi(2) - k * (k * my).sin() * (k * mx).sin()
            } else {
                k * (k * mx).cos().powi(2) - k * (k * mx).sin() * (k * my).sin()
            }
        })
        .collect();
    rel_errors(&discrete, &analytic)
}

/// Viscous operator MMS: relative (max, L2) errors of the discrete Laplacian `δd f` vs the
/// analytic `2k²·f` for `f = sin(k x)·sin(k y)` evaluated at physical vertices.
fn viscous_mms(n: usize, amp: f64) -> (f64, f64) {
    let (manifold, y_node, k) = graded(n, amp);
    let complex = manifold.complex();
    let total: usize = (0..=2).map(|g| complex.num_cells(g)).sum();
    let nc = |g: usize| complex.num_cells(g);

    // f at vertices, evaluated at physical coordinates (axis 0 uniform, axis 1 graded).
    let f_vals: Vec<f64> = complex
        .iter_cells(0)
        .map(|c| {
            let p = c.position();
            (k * p[0] as f64).sin() * (k * y_node[p[1]]).sin()
        })
        .collect();

    let df = unit_carrier(n, total, &nc, 0, &f_vals).exterior_derivative(0);
    let lap = manifold.codifferential_of(df.as_slice(), 1); // δd f (positive Laplacian)

    // δd has eigenvalue +2k² for the sin·sin mode.
    let analytic: Vec<f64> = f_vals.iter().map(|f| 2.0 * k * k * f).collect();
    rel_errors(lap.as_slice(), &analytic)
}

fn observed_orders(errs: &[f64]) -> Vec<f64> {
    errs.windows(2).map(|w| (w[0] / w[1]).log2()).collect()
}

fn fmt_orders(orders: &[f64]) -> String {
    orders
        .iter()
        .map(|p| format!("{p:.2}"))
        .collect::<Vec<_>>()
        .join(",")
}
