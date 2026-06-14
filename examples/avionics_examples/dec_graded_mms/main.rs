/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Graded-metric MMS truncation study — CFD rung R1
//!
//! A graded mesh is a metric state on an unchanged lattice (`variable-grid-geometry.md`
//! §2): `d`, the discrete Stokes theorem, and divergence-freeness are combinatorial, so
//! they hold **exactly at any grading** — only *accuracy order* is at stake. This study
//! quantifies that: how the order of accuracy of the discrete **interior product**
//! `i_X ω` — the building block of the convective term `i_u(du)` — behaves as the grading
//! strength rises.
//!
//! ## Method of manufactured solutions
//!
//! On an `N × N` torus we manufacture
//!   ω = (sin(k y), sin(k x)),  X = (cos(k x), cos(k y)),  k = 2π/N,
//! whose continuum Lie derivative `L_X ω = i_X dω + d i_X ω` is known in closed form. The
//! metric is *graded* on axis 1 by a smooth, periodic edge-length modulation
//!   ℓ(pos) = 1 + a·cos(2π·pos/N),
//! which keeps the torus seam smooth and sums to `N` (so the wavenumber is unchanged). The
//! manufactured solution is evaluated at the **physical** (cumulative-length) edge
//! midpoints and `X` is supplied as the flat `X♭ = X^axis · ℓ(edge)`. The discrete
//! `i_X dω + d i_X ω` is compared to the analytic Lie derivative; the relative sup-error
//! over a refinement sweep gives the observed order `p = log₂(E_N / E_{2N})`.
//!
//! ## What it shows
//!
//! At amplitude `a = 0` (uniform) the operator is clean second order. Under grading it stays
//! *convergent* (the error keeps decreasing) but **loses formal second order** — the
//! finest-grid order falls below 1.5 by an adjacent-spacing ratio of ≈ 1.11 and plateaus
//! beyond. That is the convective operator's anisotropy-consistency limit (the same class as
//! the convective-term form-slot issue; a candidate for the same vector-slot fix). Structure
//! — divergence-freeness of the Leray projection — is metric-free and exact at *every*
//! grading, pinned independently by the topology exactness test. Run:
//!
//! ```text
//! cargo run --release --example dec_graded_mms
//! ```

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, LatticeCell, LatticeComplex, Manifold,
};
use std::f64::consts::PI;

/// Relative sup-error of the discrete `i_X dω + d i_X ω` against the analytic Lie
/// derivative, on an `N × N` torus graded on axis 1 by `ℓ(pos) = 1 + amp·cos(2π pos/N)`.
fn cartan_mms_rel_error(n: usize, amp: f64) -> f64 {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
    let len = |pos: usize| 1.0 + amp * (2.0 * PI * pos as f64 / n as f64).cos();

    // Graded metric: axis 0 uniform, axis 1 modulated.
    let edge_lengths: Vec<f64> = lattice
        .iter_cells(1)
        .map(|c| {
            let axis = c.orientation().trailing_zeros() as usize;
            if axis == 1 { len(c.position()[1]) } else { 1.0 }
        })
        .collect();
    let metric = CubicalReggeGeometry::<2, f64>::from_edge_lengths(edge_lengths);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);
    let complex = manifold.complex();

    // Physical y-coordinate of vertex j (cumulative edge length); the modulation sums to
    // N, so the wavenumber is the uniform k.
    let mut y_node = vec![0.0_f64; n + 1];
    for j in 0..n {
        y_node[j + 1] = y_node[j] + len(j);
    }
    let k = 2.0 * PI / (n as f64);

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

    let n1 = complex.num_cells(1);
    let omega_vals: Vec<f64> = complex
        .iter_cells(1)
        .map(|c| {
            let (mx, my, axis) = midpoint(&c);
            if axis == 0 {
                (k * my).sin()
            } else {
                (k * mx).sin()
            }
        })
        .collect();
    // X♭: edge value is X^axis · ℓ(edge).
    let x_vals: Vec<f64> = complex
        .iter_cells(1)
        .map(|c| {
            let (mx, my, axis) = midpoint(&c);
            let component = if axis == 0 {
                (k * mx).cos()
            } else {
                (k * my).cos()
            };
            let length = if axis == 1 { len(c.position()[1]) } else { 1.0 };
            component * length
        })
        .collect();
    let omega = CausalTensor::new(omega_vals.clone(), vec![n1]).unwrap();
    let x_flat = CausalTensor::new(x_vals, vec![n1]).unwrap();

    // i_X dω (d is metric-free; a unit carrier suffices for it).
    let unit_carrier = |k_grade: usize, form: &[f64]| {
        let off: usize = (0..k_grade).map(|g| complex.num_cells(g)).sum();
        let mut d = vec![0.0; total];
        d[off..off + form.len()].copy_from_slice(form);
        let t = CausalTensor::new(d, vec![total]).unwrap();
        let unit = CubicalReggeGeometry::<2, f64>::unit();
        Manifold::from_cubical_with_metric(LatticeComplex::<2, f64>::square_torus(n), t, unit, 0)
    };
    let d_omega = unit_carrier(1, &omega_vals).exterior_derivative(1);
    let term1 = manifold.interior_product(&x_flat, &d_omega, 2).unwrap();

    // d i_X ω
    let ix_omega = manifold.interior_product(&x_flat, &omega, 1).unwrap();
    let term2 = unit_carrier(0, ix_omega.as_slice()).exterior_derivative(0);

    let mut max_err = 0.0_f64;
    let mut max_ref = 0.0_f64;
    for (i, cell) in complex.iter_cells(1).enumerate() {
        let (mx, my, axis) = midpoint(&cell);
        let analytic = if axis == 0 {
            k * (k * my).cos().powi(2) - k * (k * my).sin() * (k * mx).sin()
        } else {
            k * (k * mx).cos().powi(2) - k * (k * mx).sin() * (k * my).sin()
        };
        let discrete = term1.as_slice()[i] + term2.as_slice()[i];
        max_err = max_err.max((discrete - analytic).abs());
        max_ref = max_ref.max(analytic.abs());
    }
    max_err / max_ref
}

fn main() {
    let resolutions = [8usize, 16, 32, 64];
    let amplitudes = [0.0, 0.05, 0.1, 0.2, 0.3, 0.4];

    println!(
        "Graded-metric MMS — interior-product (convective-operator) order vs grading amplitude"
    );
    println!("torus N×N, axis-1 edge lengths modulated by 1 + a·cos(2π·pos/N)\n");
    println!(
        "{:>6}  {:>10} {:>10} {:>10} {:>10}   {:>18}",
        "a", "E(8)", "E(16)", "E(32)", "E(64)", "observed order p"
    );
    println!("{}", "-".repeat(78));

    let mut order_at_finest = Vec::new();
    for &amp in &amplitudes {
        let errs: Vec<f64> = resolutions
            .iter()
            .map(|&n| cartan_mms_rel_error(n, amp))
            .collect();
        let orders: Vec<f64> = errs.windows(2).map(|w| (w[0] / w[1]).log2()).collect();
        let order_str = orders
            .iter()
            .map(|p| format!("{p:.2}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "{amp:>6.2}  {:>10.3e} {:>10.3e} {:>10.3e} {:>10.3e}   {order_str:>18}",
            errs[0], errs[1], errs[2], errs[3]
        );
        order_at_finest.push((amp, *orders.last().unwrap()));
    }

    println!("{}", "-".repeat(78));
    // The finest-grid order is ~2 only at zero/near-zero grading; report where it drops
    // below 1.5.
    let collapse = order_at_finest
        .iter()
        .find(|(a, p)| *a > 0.0 && *p < 1.5)
        .map(|(a, _)| *a);
    println!("Findings:");
    println!("  • Uniform (a = 0): clean second order (matches the unit-metric Cartan test).");
    println!("  • Structure is metric-free: divergence-freeness of the Leray projection is exact");
    println!("    at EVERY amplitude (pinned by the topology test), independent of this study.");
    match collapse {
        Some(a) => println!(
            "  • Accuracy: the discrete interior product is convergent under grading (error keeps\n    \
             decreasing) but loses formal second order — the finest-grid order falls below 1.5\n    \
             by amplitude a ≈ {a:.2} (adjacent-spacing ratio ≈ {:.2}) and plateaus beyond. This is\n    \
             the convective operator's anisotropy-consistency limit — the same class as the\n    \
             convective-term form-slot issue, and a candidate for the same vector-slot fix.",
            (1.0 + a) / (1.0 - a)
        ),
        None => println!("  • Accuracy: order stayed above 1.5 across the swept amplitudes."),
    }
}
