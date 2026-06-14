/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Cut-cell cylinder wake — CFD Stage 4, Group D harness
//!
//! Flow past a circular cylinder built as an **immersed cut-cell body** on a uniform lattice,
//! exercising the full Stage-4 stack end to end:
//!
//! - the cylinder geometry is a `CutCellRegistry` from the analytic disk primitive
//!   (`CutCellRegistry::from_primitive`, A4) — clipped volumes + apertures, not a staircase;
//! - the cut **Hodge star** (B5) makes every operator (compiled stencils, constrained Leray
//!   projection, codifferential) see the partial cells transparently;
//! - the immersed **no-slip / no-penetration** condition (B4) pins the body's edges through
//!   the existing constrained projector.
//!
//! ## What this harness is (and is not)
//!
//! The DEC solver's boundary conditions today are no-slip / moving walls, body force, and
//! periodicity — there is **no inflow / outflow boundary yet** (that arrives with the Stage-4
//! uncertain-inflow zone, Group C). So this drives the flow with a streamwise body force in a
//! **periodic channel** (periodic-x, wall-y) containing the cylinder: the confined /
//! periodic-array cylinder, which sheds a von-Kármán street and is a faithful exercise of the
//! cut-cell machinery. The quantitative isolated-cylinder Reynolds ladder against Lehmkuhl et
//! al. (2013) and the Williamson lineage (tasks D2/D3) needs that inflow/outflow surface plus
//! the small-cell stabilizer selection (B1–B3); it is **not** claimed here.
//!
//! ## Small-cell guard (placeholder for B1–B3)
//!
//! Arbitrarily small cut cells tighten the CFL bound catastrophically — the canonical cut-cell
//! hazard the Group-B stabilizer (cell-merging vs flux-redistribution) will resolve. Until
//! that decision lands, this harness applies a simple **volume-fraction merge**: a cut cell
//! whose wetted fraction is below `MERGE_FRACTION` is absorbed into the solid body. This is a
//! crude cell-merging stand-in (documented as such), enough to keep the march stable while the
//! real stabilizer is prototyped on this very case.
//!
//! ```text
//! cargo run --release -p avionics_examples --example dec_cylinder_wake
//! ```

use deep_causality_physics::{BodyForceOneForm, DecNsSolver};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCell, CutCellRegistry, LatticeComplex, Manifold,
    Primitive,
};

// -- Case parameters --------------------------------------------------------------------

/// Cells across the channel height (wall-normal, y).
const NY: usize = 32;
/// Channel aspect ratio (streamwise length / height); the domain is `AR·H × H`.
const AR: f64 = 3.0;
/// Cylinder diameter as a fraction of the channel height (blockage ratio).
const BLOCKAGE: f64 = 0.25;
/// Reynolds number based on the cylinder diameter and the target bulk velocity.
const RE_D: f64 = 100.0;
/// Target bulk streamwise velocity (sets the body-force magnitude).
const U_BULK: f64 = 1.0;
/// Cut cells wetter than this fraction are kept; below it they merge into the body
/// (small-cell guard — placeholder for the B1–B3 stabilizer).
const MERGE_FRACTION: f64 = 0.5;
/// Number of march steps. The harness demonstrates a stable, divergence-free cut-cell march;
/// developed shedding needs a longer run (and ideally the inflow/outflow surface) — see D2/D3.
const STEPS: usize = 2000;

fn main() {
    let h = 1.0 / (NY - 1) as f64; // channel height H = 1.
    let nx = (AR / h).round() as usize;
    let lattice = LatticeComplex::<2, f64>::new([nx, NY], [true, false]); // periodic-x, wall-y.

    let diameter = BLOCKAGE; // H = 1, so D = BLOCKAGE·H.
    let radius = 0.5 * diameter;
    let center = [AR * 0.25, 0.5]; // a quarter-length in, mid-channel.
    let nu = U_BULK * diameter / RE_D;

    // Cut geometry: the analytic disk, then the small-cell merge guard.
    let base_metric = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball(center, radius);
    let raw =
        CutCellRegistry::from_primitive(&lattice, &base_metric, &disk).expect("disk intersection");
    let (registry, n_solid, n_cut, n_merged) = guard_small_cells(&raw, h * h);

    let fluid_area = AR * 1.0 - solid_area(&registry, h * h);
    let metric = base_metric.with_cut_cells(registry);

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    // Streamwise body force on x-edges (edge integral g·h) sized so the inviscid forcing
    // balances a Poiseuille-like bulk speed; the cylinder then sheds in its wake.
    let g = 8.0 * nu * U_BULK; // Poiseuille pressure gradient for u_max ≈ U_BULK.
    let n1 = manifold.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in manifold.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = g * h;
        }
    }
    let force =
        BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), &manifold).unwrap();

    // Conservative dt: the diffusive limit on the *merged* smallest cell, with an advective
    // margin. (The proper small-cell CFL is the B3 stability rung.)
    let dt = 0.2 * h * h / (4.0 * nu);
    let solver = DecNsSolver::new(&manifold, nu, dt, Some(&force)).expect("solver");

    let n0 = manifold.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).expect("seed");

    // A wake probe: the y-velocity (transverse) one diameter downstream of the cylinder,
    // mid-channel. Its oscillation frequency is the shedding frequency.
    let probe_x = ((center[0] + 1.5 * diameter) / h).round() as usize;
    let probe_y = (0.5 / h).round() as usize;
    let probe_edge = manifold
        .complex()
        .iter_cells(1)
        .position(|c| {
            c.orientation().trailing_zeros() as usize == 1
                && c.position()[0] == probe_x.min(nx - 1)
                && c.position()[1] == probe_y.min(NY - 2)
        })
        .expect("probe edge exists");

    eprintln!(
        "# cut-cell cylinder wake: grid {nx}×{NY}, D/H={BLOCKAGE}, Re_D={RE_D}, nu={nu:.3e}, dt={dt:.3e}"
    );
    eprintln!(
        "# cells: {n_solid} solid, {n_cut} cut (kept), {n_merged} merged; fluid area {fluid_area:.4}"
    );
    println!("step,t,kinetic_energy,max_speed,div_residual,v_probe");

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
            let ke = kinetic_energy(u.as_slice(), h);
            println!(
                "{},{:.5},{:.6e},{:.6e},{:.3e},{:.6e}",
                step + 1,
                t,
                ke,
                out.max_speed(),
                out.divergence_residual(),
                v_probe,
            );
        }
        state = out.into_state();
    }

    report_strouhal(&probe_series, diameter, U_BULK);
}

/// Reclassify cut cells below `MERGE_FRACTION` wetted as solid (the small-cell guard). Returns
/// the guarded registry and `(n_solid, n_cut_kept, n_merged)` counts.
fn guard_small_cells(
    raw: &CutCellRegistry<2, f64>,
    full_area: f64,
) -> (CutCellRegistry<2, f64>, usize, usize, usize) {
    let mut out = CutCellRegistry::<2, f64>::new();
    let (mut n_solid, mut n_cut, mut n_merged) = (0usize, 0usize, 0usize);
    for (&idx, cut) in raw.iter() {
        if cut.class().is_solid() {
            out.insert(idx, cut.clone());
            n_solid += 1;
        } else if cut.volume_fraction() < MERGE_FRACTION {
            out.insert(idx, CutCell::<2, f64>::solid(full_area));
            n_merged += 1;
        } else {
            out.insert(idx, cut.clone());
            n_cut += 1;
        }
    }
    (out, n_solid, n_cut, n_merged)
}

/// Total solid area recorded in the registry (solid cells full; cut cells their dry part).
fn solid_area(registry: &CutCellRegistry<2, f64>, full_area: f64) -> f64 {
    registry
        .iter()
        .map(|(_, cut)| {
            if cut.class().is_solid() {
                full_area
            } else {
                cut.full_volume() - cut.fluid_volume()
            }
        })
        .sum()
}

/// Kinetic energy proxy `½ Σ (u_e / h)²` over the edge cochain (uniform-metric form).
fn kinetic_energy(u: &[f64], h: f64) -> f64 {
    0.5 * u.iter().map(|e| (e / h) * (e / h)).sum::<f64>()
}

/// Estimate the shedding Strouhal `St = f·D / U` from the wake probe's mean-crossing rate over
/// the developed (second-half) signal.
fn report_strouhal(series: &[(f64, f64)], diameter: f64, u_ref: f64) {
    if series.len() < 8 {
        eprintln!("# Strouhal: insufficient samples");
        return;
    }
    let half = series.len() / 2;
    let tail = &series[half..];
    let mean = tail.iter().map(|(_, v)| *v).sum::<f64>() / tail.len() as f64;

    // Up-crossings of the mean ⇒ one per period.
    let mut crossings: Vec<f64> = Vec::new();
    for w in tail.windows(2) {
        let (t0, v0) = w[0];
        let (t1, v1) = w[1];
        if v0 - mean <= 0.0 && v1 - mean > 0.0 {
            let frac = (mean - v0) / (v1 - v0);
            crossings.push(t0 + frac * (t1 - t0));
        }
    }
    if crossings.len() < 2 {
        eprintln!("# Strouhal: no clear shedding detected in the developed signal");
        return;
    }
    let period = (crossings.last().unwrap() - crossings[0]) / (crossings.len() - 1) as f64;
    let freq = 1.0 / period;
    let strouhal = freq * diameter / u_ref;
    eprintln!(
        "# shedding: period {period:.4}, f {freq:.4}, St = f·D/U ≈ {strouhal:.4} \
         (confined/periodic cylinder — not the isolated-cylinder reference)"
    );
}
