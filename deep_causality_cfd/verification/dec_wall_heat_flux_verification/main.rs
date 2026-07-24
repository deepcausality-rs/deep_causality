/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! **Fourier-law wall heat flux verification** (`add-dec-scalar-transport-wall-heat-flux`).
//!
//! `wall_heat_flux` computes `q = −k ∮_S ∇T·n dA` over an immersed body's cut-cell fragments. This
//! harness gates it against an analytic reference and against the physics of a marched field.
//!
//! The primary reference is **exact, not tolerated**. Under a linear temperature profile the
//! one-sided wall-normal reconstruction `(T_sample − T_w)/Δh` reproduces `∂T/∂n` with no truncation
//! error, so `q = −k·G·A` holds to machine precision — and holds at *every* resolution, which is
//! what gate 2 checks. A reconstruction that is accidentally tuned to one spacing cannot pass both.
//!
//! Gates:
//!   WHF-1  **exactness** — the linear-profile flux equals `−k·G·A` to the f64 floor  `[reference]`
//!   WHF-2  **resolution independence** — exact at every rung of a spacing ladder     `[reference]`
//!   WHF-3  **sign convention** — a body hotter than the fluid reports `q < 0`        `[reference]`
//!   WHF-4  **marched decay** — the wall flux out of an isothermal body decays
//!          monotonically as the thermal layer thickens                               `[tripwire]`
//!
//! WHF-4 is the one that exercises the scalar solver rather than a manufactured field: it marches
//! diffusion from an isothermal body into cold fluid and checks the flux behaves as conduction must.
//! It is a tripwire, not a reference — the decay *rate* is not compared to a closed form here.

use deep_causality_cfd::{DecScalarRate, EvidenceClass, wall_heat_flux};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCell, CutCellRegistry, CutFaceFragment, LatticeComplex,
    Manifold, SourceGeometry,
};

const N: usize = 16;
const AREA: f64 = 1.0;
const K: f64 = 3.0;
const GRADIENT: f64 = 2.0;
const T_WALL: f64 = 10.0;

/// A lattice of spacing `h` carrying one cut cell with a single planar `+x` fragment.
fn plate(
    h: f64,
) -> (
    Manifold<LatticeComplex<2, f64>, f64>,
    CutCellRegistry<2, f64>,
) {
    let lattice = LatticeComplex::<2, f64>::new([N, N], [false, false]);
    let cells: Vec<_> = lattice.iter_cells(2).collect();
    let base = [3usize, 3usize];
    let cell_id = cells
        .iter()
        .position(|c| *c.position() == base)
        .expect("base cell exists");

    // Fragment geometry in **physical** lattice coordinates (`index · h`).
    let fragment =
        CutFaceFragment::<2, f64>::new(AREA, [1.0, 0.0], [4.0 * h, 3.5 * h], SourceGeometry::Plane);
    let mut registry = CutCellRegistry::<2, f64>::new();
    registry.insert(
        cell_id,
        CutCell::<2, f64>::cut(1.0, 0.5, [[1.0, 1.0], [1.0, 1.0]], vec![fragment]),
    );

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(h);
    (
        Manifold::from_cubical_with_metric(lattice, data, metric, 0),
        registry,
    )
}

/// `T(x, y) = T_w + G·(x − x_wall)`, linear in the wall normal.
fn linear_field(
    m: &Manifold<LatticeComplex<2, f64>, f64>,
    h: f64,
    gradient: f64,
    x_wall: f64,
) -> CausalTensor<f64> {
    let vals: Vec<f64> = m
        .complex()
        .iter_cells(0)
        .map(|v| T_WALL + gradient * (v.position()[0] as f64 * h - x_wall))
        .collect();
    let n = vals.len();
    CausalTensor::new(vals, vec![n]).unwrap()
}

fn main() {
    println!("# DEC Fourier-law wall heat flux — q = -k ∮ ∇T·n dA over cut-cell fragments");
    println!("# reference: a linear profile makes the one-sided reconstruction exact, so");
    println!("#            q = -k·G·A analytically, at every resolution.\n");

    let mut failed = false;
    let mut gate = |label: &str, pass: bool, evidence: EvidenceClass, detail: String| {
        println!(
            "  [{}] [{evidence}] {label}: {detail}",
            if pass { "PASS" } else { "FAIL" }
        );
        if !pass {
            failed = true;
        }
    };

    // --- WHF-1: exactness at the nominal spacing -------------------------------------------------
    let h0 = 0.25;
    let (m, reg) = plate(h0);
    let t = linear_field(&m, h0, GRADIENT, 4.0 * h0);
    let q = wall_heat_flux(&m, &reg, &t, T_WALL, K).unwrap();
    let analytic = -K * GRADIENT * AREA;
    gate(
        "WHF-1 linear profile reproduces Fourier's law",
        (q - analytic).abs() < 1e-12,
        EvidenceClass::Reference,
        format!("q = {q:.12} vs analytic -k·G·A = {analytic:.12}"),
    );

    // --- WHF-2: exact at every spacing -----------------------------------------------------------
    // A reconstruction that mis-scales Δh is exact at h = 1 and wrong elsewhere, so a ladder that
    // includes h = 1 alongside other spacings separates "correct" from "accidentally correct".
    println!("\n# WHF-2 resolution ladder (analytic value is spacing-independent)");
    println!("  {:>8} | {:>18} | {:>12}", "h", "q", "|q - analytic|");
    let mut worst = 0.0f64;
    for &h in &[1.0, 0.5, 0.25, 0.125, 0.0625] {
        let (m, reg) = plate(h);
        let t = linear_field(&m, h, GRADIENT, 4.0 * h);
        let qh = wall_heat_flux(&m, &reg, &t, T_WALL, K).unwrap();
        let err = (qh - analytic).abs();
        worst = worst.max(err);
        println!("  {h:>8.4} | {qh:>18.12} | {err:>12.3e}");
    }
    gate(
        "WHF-2 exact at every spacing",
        worst < 1e-12,
        EvidenceClass::Reference,
        format!("worst |q - analytic| over the ladder = {worst:.3e}"),
    );

    // --- WHF-3: sign convention ------------------------------------------------------------------
    let t_cold = linear_field(&m, h0, -GRADIENT, 4.0 * h0);
    let q_cold = wall_heat_flux(&m, &reg, &t_cold, T_WALL, K).unwrap();
    gate(
        "WHF-3 sign convention (outward n; q > 0 leaves the wall)",
        q_cold > 0.0 && q < 0.0 && (q + q_cold).abs() < 1e-12,
        EvidenceClass::Reference,
        format!("cold fluid q = {q_cold:.6} > 0, hot fluid q = {q:.6} < 0"),
    );

    // --- WHF-4: a marched field behaves like conduction ------------------------------------------
    // Everything above uses a manufactured field. This rung marches the scalar solver: an isothermal
    // body diffusing into cold fluid must show the wall flux decay as the thermal layer thickens.
    let (mm, mreg) = plate(h0);
    let solid_reg = {
        let lattice = LatticeComplex::<2, f64>::new([N, N], [false, false]);
        let cells: Vec<_> = lattice.iter_cells(2).collect();
        let mut r = CutCellRegistry::<2, f64>::new();
        for base in [[3usize, 3usize]] {
            if let Some(id) = cells.iter().position(|c| *c.position() == base) {
                r.insert(id, CutCell::<2, f64>::solid(1.0));
            }
        }
        r
    };
    let kappa = 0.05;
    let rate = DecScalarRate::new(&mm, kappa)
        .unwrap()
        .with_isothermal_body(&solid_reg, T_WALL)
        .unwrap();
    let n1 = mm.complex().num_cells(1);
    let u = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let n0 = mm.complex().num_cells(0);
    let mut field = CausalTensor::new(vec![0.0; n0], vec![n0]).unwrap();
    rate.apply_wall(&mut field.as_slice().to_vec());

    println!("\n# WHF-4 marched conduction from an isothermal body (kappa = {kappa})");
    println!("  {:>6} | {:>18}", "step", "wall flux");
    let dt = 0.2 * h0 * h0 / (4.0 * kappa);
    let mut fluxes: Vec<f64> = Vec::new();
    for s in 0..=40 {
        if s > 0 {
            field = rate.step(&field, &u, dt).unwrap();
        }
        if s % 10 == 0 && s > 0 {
            let qs = wall_heat_flux(&mm, &mreg, &field, T_WALL, K).unwrap();
            println!("  {s:>6} | {qs:>18.9}");
            fluxes.push(qs);
        }
    }
    let monotone = fluxes.windows(2).all(|w| w[1].abs() <= w[0].abs() + 1e-12);
    let nonzero = fluxes.first().map(|f| f.abs() > 1e-9).unwrap_or(false);
    gate(
        "WHF-4 marched wall flux decays as the thermal layer thickens",
        monotone && nonzero,
        EvidenceClass::Tripwire,
        format!(
            "|q| over the march: {:?} (monotone {monotone}, non-trivial {nonzero})",
            fluxes.iter().map(|f| f.abs()).collect::<Vec<_>>()
        ),
    );

    println!("\n--- reading ---");
    println!(
        "  The flux is a genuine surface integral: a gradient, a conductivity and a wall normal"
    );
    println!(
        "  integrated over fragment area. It is exact on a linear profile at every spacing, so"
    );
    println!(
        "  the wall-normal step and the area weighting are both carried correctly — neither is"
    );
    println!(
        "  absorbed into a tolerance. This is the quantity `penalization_heat_integral` is not."
    );

    if failed {
        eprintln!("\nFAILED GATES — see [FAIL] lines above.");
        std::process::exit(1);
    }
    println!("\nALL GATES PASSED — Fourier's law holds exactly on the reference profile.");
}
