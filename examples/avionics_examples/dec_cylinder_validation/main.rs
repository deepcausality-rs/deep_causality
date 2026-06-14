/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Isolated cylinder — CFD Stage 4 D2/D3 validation harness
//!
//! Flow past an isolated circular cylinder, assembled from the full Stage-4 boundary-zone stack:
//!
//! - **west `Inflow`** (uniform wall-normal Dirichlet `U`),
//! - **east `Outflow`** (pressure-reference, zero-gradient — the net-flux open-boundary projection),
//! - **far-field `SlipWall` top/bottom** (no penetration, zero shear — so the lateral boundaries do
//!   not confine the wake), and
//! - the **immersed cut cylinder** (`CutCellRegistry::from_primitive`, exact clipped volumes +
//!   apertures; its no-slip is the auto-derived solid-incident set).
//!
//! This is the external-flow domain the confined/periodic harness could not express; it is the
//! configuration the Williamson (Strouhal) and Lehmkuhl et al. 2013 (drag) references describe.
//!
//! ## What this rung reports
//!
//! The wake probe (transverse velocity ~1.5 D downstream) gives the shedding **Strouhal**
//! `St = f·D/U`. Reference: Williamson `St(Re=100) ≈ 0.164`. The harness streams the probe and
//! prints the Strouhal estimate over the developed signal.
//!
//! Drag `C_d` needs the **viscous (friction) traction** added to `pressure_surface_force` (~35% of
//! `C_d` at Re=100) — a follow-on; the pressure-drag scaffold is in `surface_force`.
//!
//! ## Scope
//!
//! 2D laminar (Re ≈ 100–200) is the validated regime. The 3D-transition rung (Re ≈ 200–300) and
//! Re ≈ 3900 by DNS are `const D`-ready but compute-bound (run with a larger grid / longer time).
//!
//! ```text
//! cargo run --release -p avionics_examples --example dec_cylinder_validation
//! ```

use deep_causality_physics::{DecNsSolver, Inflow, Outflow, SlipWall};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCellRegistry, LatticeComplex, Manifold, Primitive,
};

// Case parameters (kept modest so the smoke run is affordable; raise for a reference comparison).
const RE_D: f64 = 100.0;
const U: f64 = 1.0;
/// Cells across one cylinder diameter (resolution). Coarse by default so the harness runs quickly;
/// raise (with a larger domain and `STEPS`) for a reference-quality Strouhal.
const CELLS_PER_D: usize = 8;
/// Domain extent in diameters: streamwise (x) and cross-stream (y).
const LX_D: f64 = 10.0;
const LY_D: f64 = 6.0;
/// Cylinder center, in diameters from the inlet / bottom.
const CX_D: f64 = 3.0;
const CY_D: f64 = 3.0;
const MERGE_FRACTION: f64 = 0.25;
const STEPS: usize = 1500;

fn main() {
    let diameter = 1.0_f64;
    let radius = 0.5 * diameter;
    let h = diameter / CELLS_PER_D as f64;
    let nx = (LX_D / h).round() as usize;
    let ny = (LY_D / h).round() as usize;
    let nu = U * diameter / RE_D;

    // x: inflow (west) / outflow (east); y: far-field slip walls.
    let lattice = LatticeComplex::<2, f64>::new([nx, ny], [false, false]);
    let center = [CX_D, CY_D];
    let base = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball(center, radius);
    let registry = CutCellRegistry::from_primitive(&lattice, &base, &disk)
        .expect("disk intersection")
        .with_cell_merging(MERGE_FRACTION);
    let n_solid = registry
        .iter()
        .filter(|(_, c)| c.class().is_solid())
        .count();
    let n_cut = registry.iter().filter(|(_, c)| c.class().is_cut()).count();
    let metric = base.with_cut_cells(registry);

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    // Conservative dt: the advective limit at the inflow speed, with a margin.
    let dt = 0.4 * h / U;

    // The full isolated-cylinder boundary-zone set (static tuple composition).
    let zones = (
        Inflow::<2, f64>::new(0, false, U).expect("inflow"),
        (
            Outflow::<2>::new(0, true).expect("outflow"),
            (
                SlipWall::<2>::new(1, false).expect("slip bottom"),
                SlipWall::<2>::new(1, true).expect("slip top"),
            ),
        ),
    );
    let solver = DecNsSolver::with_zones(&manifold, nu, dt, zones).expect("solver");

    let n0 = manifold.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).expect("seed");

    // Wake probe: transverse (y) velocity ~1.5 D downstream of the cylinder, mid-channel.
    let probe_x = ((center[0] + 1.5 * diameter) / h).round() as usize;
    let probe_y = (center[1] / h).round() as usize;
    let probe_edge = manifold
        .complex()
        .iter_cells(1)
        .position(|c| {
            c.orientation().trailing_zeros() as usize == 1
                && c.position()[0] == probe_x.min(nx - 1)
                && c.position()[1] == probe_y.min(ny - 2)
        })
        .expect("probe edge exists");

    eprintln!(
        "# isolated cylinder: grid {nx}×{ny} ({CELLS_PER_D}/D), domain {LX_D}×{LY_D} D, Re_D={RE_D}, nu={nu:.3e}, dt={dt:.3e}"
    );
    eprintln!("# cut cells: {n_solid} solid, {n_cut} cut; merge floor {MERGE_FRACTION}");
    println!("step,t,max_speed,interior_div,v_probe");

    let mut probe_series: Vec<(f64, f64)> = Vec::with_capacity(STEPS);
    let report_every = (STEPS / 200).max(1);
    for step in 0..STEPS {
        let out = match solver.step(&state) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("# march stopped at step {step}: {e}");
                break;
            }
        };
        let t = (step + 1) as f64 * dt;
        let u = out.state().as_one_form();
        let v_probe = u.as_slice()[probe_edge] / h;
        probe_series.push((t, v_probe));
        if (step + 1) % report_every == 0 {
            // The global divergence residual includes the open inlet/outlet (where the boundary
            // flux makes δu nonzero by design); the *interior* divergence is the meaningful check.
            let codiff = manifold.codifferential_of(u.as_slice(), 1).into_vec();
            let interior_div = manifold
                .complex()
                .iter_cells(0)
                .enumerate()
                .filter_map(|(i, c)| {
                    let p = c.position();
                    (p[0] > 0 && p[0] + 1 < nx && p[1] > 0 && p[1] + 1 < ny)
                        .then_some(codiff[i].abs())
                })
                .fold(0.0_f64, f64::max);
            println!(
                "{},{:.4},{:.5e},{:.2e},{:.6e}",
                step + 1,
                t,
                out.max_speed(),
                interior_div,
                v_probe,
            );
        }
        state = out.into_state();
    }

    report_strouhal(&probe_series, diameter, U);
}

/// Estimate `St = f·D/U` from the wake probe's mean-crossing rate over the developed (second-half)
/// signal, and compare to the Williamson isolated-cylinder reference at Re = 100.
fn report_strouhal(series: &[(f64, f64)], diameter: f64, u_ref: f64) {
    if series.len() < 16 {
        eprintln!("# Strouhal: insufficient samples");
        return;
    }
    let tail = &series[series.len() / 2..];
    let mean = tail.iter().map(|(_, v)| *v).sum::<f64>() / tail.len() as f64;
    let mut crossings: Vec<f64> = Vec::new();
    for w in tail.windows(2) {
        let (t0, v0) = w[0];
        let (t1, v1) = w[1];
        if v0 - mean <= 0.0 && v1 - mean > 0.0 {
            crossings.push(t0 + (mean - v0) / (v1 - v0) * (t1 - t0));
        }
    }
    if crossings.len() < 2 {
        eprintln!("# Strouhal: no clear shedding detected yet (run longer / refine the grid)");
        return;
    }
    let period = (crossings.last().unwrap() - crossings[0]) / (crossings.len() - 1) as f64;
    let st = (1.0 / period) * diameter / u_ref;
    eprintln!("# shedding: period {period:.4}, St = f·D/U ≈ {st:.4}  (Williamson Re=100 ≈ 0.164)");
}
