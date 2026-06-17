/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Cut-cell cylinder wake — CFD Stage 4, Groups C + D
//!
//! Flow past a circular cylinder built as an **immersed cut-cell body** (Group D), driven by a
//! **sensor-fed uncertain inflow** through a **causal-monad march** (Group C). It exercises the
//! full Stage-4 stack end to end:
//!
//! - the cylinder geometry is a `CutCellRegistry` from the analytic disk primitive
//!   (`CutCellRegistry::from_primitive`, A4) — clipped volumes + apertures, not a staircase;
//! - the cut **Hodge star** (B5) makes every operator (compiled stencils, constrained Leray
//!   projection, codifferential) see the partial cells transparently;
//! - the immersed **no-slip / no-penetration** condition (B4) pins the body's edges through the
//!   existing constrained projector;
//! - the channel is driven by a **moving top wall whose velocity is a `MaybeUncertain<f64>`
//!   sensor stream** (Group C): each step the reading is presence-gated and collapsed to a scalar
//!   inflow; a **dropout** falls back to the last-good value through a Pearl `do(...)`
//!   intervention, recorded in the `EffectLog`.
//!
//! ## The causal-monad march (Group C)
//!
//! The solver is **stateless and portable** (`step(&self, field)`); the **state lives in the
//! monad**. Each step is the `inflow_march_step` bind stage over a
//! `PropagatingProcess<f64, InflowMarchState, InflowContext>`: it collapses the sensor sample to a
//! prescribed wall velocity, reconfigures the boundary through the existing moving-wall lift, and
//! marches — the uncertain types never enter the solver core. Here we drive the march one bind at
//! a time so the per-step wake probe can be streamed; `deep_causality_cfd::march_inflow` packages
//! the same stage as a `CausalFlow::iterate_n` loop for the fire-and-report case.
//!
//! ## What this harness is (and is not)
//!
//! The DEC solver has no inflow/outflow surface; the sensor drives a **prescribed moving wall** (a
//! Dirichlet boundary the solver already supports), confined in a **periodic-x channel**. This
//! sheds a von-Kármán street and is a faithful exercise of the cut-cell + uncertain-zone
//! machinery. The quantitative isolated-cylinder Reynolds ladder against Lehmkuhl et al. (2013)
//! and the Williamson lineage (tasks D2/D3) needs that inflow/outflow surface; it is **not**
//! claimed here.
//!
//! ## Small-cell stabilization (B1/B2)
//!
//! In a finite-volume cut-cell solver, arbitrarily small cut cells collapse the explicit time
//! step (the canonical hazard). **Not here:** the cut Hodge star is a consistent metric clip, so
//! the codifferential `δ = M⁻¹ ∂ M` cancels it across grades and the explicit march is inherently
//! small-cell-stable (design D4 / `cut_cell_wiring_tests`). The selected stabilizer is therefore
//! **cell-merging** (`CutCellRegistry::with_cell_merging`), engaged here only to tighten the
//! masked-CG projection conditioning on sliver cells.
//!
//! ## Reproducibility
//!
//! The run is **bit-identical** across invocations. The sensor resolution is split into a
//! Monte-Carlo presence gate and a Quasi-Monte-Carlo collapse, and three variation sources are
//! pinned:
//!
//! - **Presence gate (Monte-Carlo).** The SPRT `to_bool` test draws from the thread RNG; the run
//!   seeds it (`seed_sampler`) to fix that realization. (QMC is invalid for the sequential test.)
//! - **Collapse (Quasi-Monte-Carlo).** The present reading's mean is estimated with
//!   `with_qmc_collapse` — Sobol + inverse-CDF — instead of plain Monte-Carlo, cutting the
//!   estimator variance at the same sample count. It is deterministic by construction (the per-step
//!   Sobol shift is `base ⊕ sample.id()`), so it needs no RNG seed.
//! - **Cut-cell order.** The registry is built with `with_deterministic_order`, so its cells iterate
//!   in ascending cell-id order rather than the per-process `HashMap` hasher order. Without it the
//!   cut-face constraint rows — hence the constrained projection's floating-point summation order —
//!   permute every run, a ~1e-11 roundoff drift (the divergence residual stays ~1e-15, so it is
//!   physically irrelevant; the deterministic order simply makes it reproducible too).
//!
//! ```text
//! cargo run --release -p avionics_examples --example dec_cylinder_wake
//! ```

use deep_causality_cfd::{CfdConfigBuilder, CfdFlow, DropoutVerbosity, UncertainInflowZone};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCellRegistry, LatticeComplex, Manifold, Primitive,
};
use deep_causality_uncertain::{MaybeUncertain, Uncertain, seed_sampler};

// -- Case parameters --------------------------------------------------------------------

/// Cells across the channel height (wall-normal, y).
const NY: usize = 32;
/// Channel aspect ratio (streamwise length / height); the domain is `AR·H × H`.
const AR: f64 = 3.0;
/// Cylinder diameter as a fraction of the channel height (blockage ratio).
const BLOCKAGE: f64 = 0.25;
/// Reynolds number based on the cylinder diameter and the target bulk velocity.
const RE_D: f64 = 100.0;
/// Target bulk streamwise velocity — the sensor's nominal reading and the moving-wall lid speed.
const U_BULK: f64 = 1.0;
/// Cell-merging floor (B1/B2): free cut cells/edges below this wetted fraction borrow volume to
/// reach it, tightening the masked-CG projection conditioning. (Explicit stability is inherent
/// here — design D4 — so this is not a CFL guard.)
const MERGE_FRACTION: f64 = 0.25;
/// Number of march steps.
const STEPS: usize = 2000;

// -- Sensor-stream parameters (Group C) -------------------------------------------------

/// Relative 1σ noise on a *present* inflow reading.
const SENSOR_SIGMA: f64 = 0.03;
/// A sensor dropout (absent reading) every this many steps — exercises the BC-fallback
/// intervention and its `EffectLog` record.
const DROPOUT_EVERY: usize = 50;
/// Sample budgets: the Monte-Carlo presence gate and the Quasi-Monte-Carlo mean collapse.
const PRESENCE_SAMPLES: usize = 256;
const COLLAPSE_SAMPLES: usize = 256;
/// Seed for the Monte-Carlo presence gate (SPRT `to_bool`). Seeding the thread-local sampler RNG
/// fixes the gate's realization so the run is reproducible byte-for-byte instead of drawing fresh
/// OS entropy each time.
const SAMPLER_SEED: u64 = 0x5EED_C0DE;
/// Base seed for the **Quasi-Monte-Carlo** collapse of the present sensor reading. QMC (Sobol +
/// inverse-CDF) estimates the per-step mean with far lower variance than plain Monte-Carlo at the
/// same `COLLAPSE_SAMPLES`; the per-step Sobol shift is `base ⊕ sample.id()`, so the collapse is
/// reproducible and independent across steps. (The presence gate stays Monte-Carlo — QMC is invalid
/// for the sequential SPRT.)
const QMC_COLLAPSE_SEED: u64 = 0x0B0_5E11;

fn main() {
    // Pin the sensor realization so the wake CSV is reproducible across runs (the only randomness
    // is the `Uncertain` sensor sampling; the DEC solver and geometry are deterministic).
    seed_sampler(SAMPLER_SEED);

    let h = 1.0 / (NY - 1) as f64; // channel height H = 1.
    let nx = (AR / h).round() as usize;
    let lattice = LatticeComplex::<2, f64>::new([nx, NY], [true, false]); // periodic-x, wall-y.

    let diameter = BLOCKAGE; // H = 1, so D = BLOCKAGE·H.
    let radius = 0.5 * diameter;
    let center = [AR * 0.25, 0.5]; // a quarter-length in, mid-channel.
    let nu = U_BULK * diameter / RE_D;

    // Cut geometry: the analytic disk, with cell-merging small-cell stabilization (B1/B2).
    let base_metric = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball(center, radius);
    // `with_deterministic_order` pins cut-cell iteration to ascending cell id, so the cut-face
    // constraint rows — hence the constrained projection's summation order — are reproducible run
    // to run (otherwise the per-process `HashMap` hasher permutes them, a ~1e-11 roundoff drift).
    let registry = CutCellRegistry::from_primitive(&lattice, &base_metric, &disk)
        .expect("disk intersection")
        .with_deterministic_order();
    let n_solid = registry
        .iter()
        .filter(|(_, c)| c.class().is_solid())
        .count();
    let n_cut = registry.iter().filter(|(_, c)| c.class().is_cut()).count();
    let fluid_area = AR * 1.0 - solid_area(&registry, h * h);

    let registry = registry.with_cell_merging(MERGE_FRACTION);
    let metric = base_metric.with_cut_cells(registry);

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    // Conservative dt: the diffusive limit, with an advective margin for the lid-driven bulk.
    let dt = 0.2 * h * h / (4.0 * nu);

    // A wake probe: the y-velocity (transverse) one diameter downstream of the cylinder.
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

    // Group C: the immutable zone + the per-step sensor stream (the only `MaybeUncertain` memory).
    let zone = UncertainInflowZone::new(1, true, 0, U_BULK)
        .with_presence_gate(0.5, 0.95, 0.05, PRESENCE_SAMPLES)
        .with_collapse_samples(COLLAPSE_SAMPLES)
        .with_qmc_collapse(QMC_COLLAPSE_SEED)
        .with_verbosity(DropoutVerbosity::EachDropout);
    let stream = sensor_stream(STEPS);
    let n_dropouts = stream
        .iter()
        .filter(|s| matches!(s.sample(), Ok(None)))
        .count();

    eprintln!(
        "# cut-cell cylinder wake: grid {nx}×{NY}, D/H={BLOCKAGE}, Re_D={RE_D}, nu={nu:.3e}, dt={dt:.3e}"
    );
    eprintln!(
        "# cells: {n_solid} solid, {n_cut} cut; fluid area {fluid_area:.4}; merge floor {MERGE_FRACTION}"
    );
    eprintln!(
        "# sensor-fed top wall: U≈{U_BULK} (1σ {SENSOR_SIGMA}), dropout every {DROPOUT_EVERY} steps ({n_dropouts} total)"
    );
    println!("step,t,kinetic_energy,max_speed,div_residual,v_probe");

    // The causal-monad march via the CfdFlow DSL: configuration (solver + sensor zone + stream +
    // horizon) is declared, the caller-owned geometry is lent with `.on(&manifold)` (B1), and
    // `run_with` streams an `UncertainStepView` per step so the wake probe can be sampled — the same
    // `inflow_march_step` bind the hand-rolled loop used (and `march_inflow` packages).
    let config = CfdConfigBuilder::uncertain_march::<f64>("cylinder-wake")
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(nu)
                .time_step(dt)
                .build()
                .expect("solver config"),
        )
        .inflow_zone(zone)
        .sensor_stream(stream)
        .march_for(STEPS)
        .build()
        .expect("uncertain-march config");

    let mut probe_series: Vec<(f64, f64)> = Vec::with_capacity(STEPS);
    let report_every = (STEPS / 200).max(1);
    let report = CfdFlow::uncertain_march(&config)
        .on(&manifold)
        .run_with(|sv| {
            let t = sv.step() as f64 * dt;
            let u = sv.one_form();
            let v_probe = u.as_slice()[probe_edge] / h;
            probe_series.push((t, v_probe));
            if sv.step() % report_every == 0 {
                let ke = kinetic_energy(u.as_slice(), h);
                let max_speed = sv.max_speed().unwrap_or(f64::NAN);
                let div = sv.divergence().unwrap_or(f64::NAN);
                println!(
                    "{},{:.5},{:.6e},{:.6e},{:.3e},{:.6e}",
                    sv.step(),
                    t,
                    ke,
                    max_speed,
                    div,
                    v_probe,
                );
            }
        })
        .expect("uncertain march");

    eprintln!(
        "# EffectLog: {} entries recorded ({n_dropouts} dropouts × (fallback + intervention))",
        report.log_entries().unwrap_or(0)
    );
    report_strouhal(&probe_series, diameter, U_BULK);
}

/// The per-step sensor stream: a noisy present reading at `U_BULK`, with a periodic dropout.
fn sensor_stream(steps: usize) -> Vec<MaybeUncertain<f64>> {
    (0..steps)
        .map(|s| {
            if (s + 1) % DROPOUT_EVERY == 0 {
                MaybeUncertain::<f64>::always_none()
            } else {
                MaybeUncertain::<f64>::from_uncertain(Uncertain::normal(U_BULK, SENSOR_SIGMA))
            }
        })
        .collect()
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
