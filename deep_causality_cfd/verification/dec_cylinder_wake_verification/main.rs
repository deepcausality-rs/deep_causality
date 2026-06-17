/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Cut-cell cylinder wake — CFD Stage 4, Groups C + D — via the CfdFlow DSL
//!
//! Flow past a circular cylinder built as an **immersed cut-cell body**, driven by a
//! **sensor-fed uncertain inflow** through the **causal-monad march**. The case is declared
//! through the `deep_causality_cfd` configuration layer ([`config`]) — the cut-cell geometry, the
//! presence-gated + QMC-collapsed inflow zone, and the sensor stream — and run through the **CfdFlow**
//! DSL: `CfdFlow::uncertain_march(&config).on(&manifold).run_with(...)` drives the per-step
//! `inflow_march_step` bind and streams an `UncertainStepView` so the wake probe can be sampled.
//! [`print_utils`] renders the diagnostics and writes the wake CSV through the IO effect.
//!
//! ## Precision is a parameter
//!
//! The whole computation — geometry, the DEC march, the uncertain inflow's working scalar, the wake
//! probe, and the Strouhal estimate — runs at the working precision [`FloatType`]; values are cast to
//! `f64` only at the display boundary (the `{:.*e}` formats). Switch the alias to re-run at another
//! precision.
//!
//! ## What this harness is (and is not)
//!
//! The DEC solver has no inflow/outflow surface; the sensor drives a **prescribed moving wall** (a
//! Dirichlet boundary), confined in a **periodic-x channel**. This sheds a von-Kármán street and is a
//! faithful exercise of the cut-cell + uncertain-zone machinery. The quantitative isolated-cylinder
//! Reynolds ladder (tasks D2/D3) needs that inflow/outflow surface; it is **not** claimed here.
//!
//! ## Reproducibility
//!
//! The run is **bit-identical** across invocations: the SPRT presence gate is seeded
//! (`seed_sampler`), the QMC collapse is deterministic by construction, and the cut-cell registry is
//! built with deterministic ordering (all in [`config`]).
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example dec_cylinder_wake_verification
//! ```

mod config;
mod print_utils;

use config::{DROPOUT_EVERY, STEPS, U_BULK, ft};
use deep_causality_cfd::{CfdFlow, fail};
use deep_causality_uncertain::seed_sampler;

/// The working precision for the whole computation (geometry, projection CG, DEC march, the uncertain
/// inflow's working scalar, and the wake-probe analysis). **This is the single alias to change**
/// (`f32`, `f64`, or `Float106`); values downcast to `f64` only at the display boundary.
pub type FloatType = f64;

fn main() {
    // Pin the sensor realization so the wake CSV is reproducible across runs.
    seed_sampler(config::SAMPLER_SEED);

    // Configuration (the "what"): the immersed cut-cell geometry and the uncertain-march container.
    let geom = config::build_geometry();
    let case = config::build_uncertain_config(geom.nu, geom.dt)
        .unwrap_or_else(|e| fail("uncertain-march configuration", e));

    // Every DROPOUT_EVERY-th step is a sensor dropout (the BC-fallback intervention + EffectLog).
    let n_dropouts = STEPS / DROPOUT_EVERY;
    print_utils::print_header(&geom, n_dropouts);

    // CfdFlow DSL; `run_with` streams the per-step wake probe. All
    // per-step quantities are native `FloatType`; the row is downcast to `f64` only for formatting.
    let (dt, h, probe_edge) = (geom.dt, geom.h, geom.probe_edge);
    let mut probe_series: Vec<(FloatType, FloatType)> = Vec::with_capacity(STEPS);
    let report = CfdFlow::uncertain_march(&case)
        .on(&geom.manifold)
        .run_with(|sv| {
            let t = ft(sv.step() as f64) * dt;
            let u = sv.one_form();
            let v_probe = u.as_slice()[probe_edge] / h;
            probe_series.push((t, v_probe));
            if sv.step() % (STEPS / 200).max(1) == 0 {
                let ke = print_utils::kinetic_energy(u.as_slice(), h);
                // Diagnostics are display-only; downcast the working-precision value directly
                // (the `.map(Into::into)` fn-pointer form trips `clippy::useless_conversion`).
                let max_speed = match sv.max_speed() {
                    Ok(v) => Into::<f64>::into(v),
                    Err(_) => f64::NAN,
                };
                let div = match sv.divergence() {
                    Ok(v) => Into::<f64>::into(v),
                    Err(_) => f64::NAN,
                };
                println!(
                    "{},{:.5},{:.6e},{max_speed:.6e},{div:.3e},{:.6e}",
                    sv.step(),
                    Into::<f64>::into(t),
                    Into::<f64>::into(ke),
                    Into::<f64>::into(v_probe),
                );
            }
        })
        .unwrap_or_else(|e| fail("uncertain march", e));

    eprintln!(
        "# EffectLog: {} entries recorded ({n_dropouts} dropouts × (fallback + intervention))",
        report.log_entries().unwrap_or(0)
    );
    print_utils::report_strouhal(&probe_series, geom.diameter, ft(U_BULK));
    print_utils::write_probe_csv("cylinder_wake.csv", &probe_series);
}
