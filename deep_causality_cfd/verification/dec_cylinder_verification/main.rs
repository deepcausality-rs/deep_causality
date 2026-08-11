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
//!   apertures; its no-slip is the **aperture-resolved** cut-face tangential constraint by default,
//!   or the staircase solid-incident set with `STAIRCASE=1` for the side-by-side comparison).
//!
//! ## Symmetry breaking — why a perturbation is needed
//!
//! The discretisation, the geometry, and the inflow are all top–bottom symmetric, so a symmetric
//! march converges to the steady symmetric wake and **never sheds**, even though the wake is
//! linearly unstable at `Re ≥ ~47`. The harness seeds a uniform stream plus a small, single-signed
//! transverse-velocity blob just downstream of the cylinder; the seed projection makes it
//! divergence-free, and it tips the flow off the symmetric branch so the von-Kármán instability can
//! grow.
//!
//! ## What this rung reports
//!
//! - The wake probe (transverse velocity ~1.5 D downstream) gives the shedding **Strouhal**
//!   `St = f·D/U`. Reference: Williamson `St(Re=100) ≈ 0.164`.
//! - The **cycle-mean drag** `C_d = F_x / (½ U² D)`, averaged over the developed (second-half)
//!   window and split into the **pressure** force (`pressure_surface_force` over the static pressure
//!   from `pressure_diagnostic`) and the **viscous (friction)** force (`viscous_surface_force`),
//!   with the lift `C_l` and the `C_d` swing. Reference: `C_d(Re=100) ≈ 1.32–1.36`, the 2-D
//!   unconfined laminar consensus (Qu et al. 2013, Posdziech & Grundmann 2007, Williamson, as
//!   compiled in arXiv:2303.09262). The directory README's gate-result section quotes the same
//!   band. Dröge & Verstappen (2005), Table II, is the secondary reference, for the
//!   pressure/friction split only: their cut-cell result is `C_d = 1.24 = 0.93` pressure `+ 0.31`
//!   friction, so friction is ≈ 25 % of `C_d`. That `1.24` is a single low-side cut-cell datum. It
//!   is not the reference band and is no longer used as its lower edge.
//!
//! ## Scope
//!
//! 2D laminar (Re ≈ 100–200) is the validated regime. The 3D-transition rung (Re ≈ 200–300) and
//! Re ≈ 3900 by DNS are `const D`-ready but compute-bound (run with a larger grid / longer time).
//! Reference-quality numbers need a finer grid and a longer run than the affordable default; raise
//! `CELLS_PER_D` and `STEPS` for a quantitative comparison.
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example dec_cylinder_verification
//! ```

use deep_causality_cfd::{
    Body, CfdConfigBuilder, CfdFlow, Inflow, Mesh, Observe, Outflow, Seed, SlipWall,
};
use deep_causality_topology::HodgeDecomposeOptions;

// Fixed case parameters. The swept parameters (Re, resolution, domain, steps) are read from the
// environment so the Re-ladder (D2/D3: Re 100–3900) and grid-refinement runs need no recompile —
// see `env_f64` / `env_usize` and the README. Defaults give an affordable laminar smoke run.
const U: f64 = 1.0;
/// Transverse-velocity seed amplitude (fraction of `U`) and Gaussian half-width (diameters), placed
/// one diameter behind the cylinder on the centerline — the symmetry-breaking trigger.
const PERTURB_EPS: f64 = 0.3;
const PERTURB_SIGMA: f64 = 0.75;

// ── Acceptance bounds ─────────────────────────────────────────────────────────────────────────
//
// Evidence class: **tripwire**, not reference. The affordable default (8 cells/D) is below
// reference-grid quality. `St` sits outside the published value: 0.1710 vs Williamson 0.164,
// +4.3 %. `C_d` lands inside the published band, 1.345 in 1.32–1.36, but for the wrong reason: the
// split is pressure 1.173 + friction 0.172, so friction is ≈ 13 % of `C_d` against the ≈ 25 % of
// the reference, and the total agrees by cancellation. Gating against the published bands at this
// resolution would fail a correctly-working solver on `St` and would reward that cancellation on
// `C_d`, so these bounds are pinned around the measured default and detect regression only. The
// published values are printed next to the measurement so the offset stays visible and is never
// read as agreement.
//
// Width is set by cross-platform floating-point sensitivity of a 1500-step nonlinear march, not by
// measurement precision (the run is deterministic on one machine). Provisional: tighten once the
// nightly CI job has established the x86_64-vs-arm64 spread.
//
// Reference condition only — the bands describe Re = 100. At any other `RE_D` the harness reports
// the measurement and states the gate is not applicable rather than passing it silently.
/// Reynolds number the published bands describe.
const REFERENCE_RE_D: f64 = 100.0;
/// Pinned Strouhal band (tripwire), ~±11 % around the measured 0.1710.
const ST_TRIPWIRE: (f64, f64) = (0.152, 0.190);
/// Pinned drag-coefficient band (tripwire), ~±10 % around the measured 1.345.
const CD_TRIPWIRE: (f64, f64) = (1.21, 1.48);
/// Published references, printed beside the measurement. Williamson (1996) for `St`. For `C_d`, the
/// 2-D unconfined laminar consensus band (Qu et al. 2013, Posdziech & Grundmann 2007, Williamson, as
/// compiled in arXiv:2303.09262); the directory README's gate-result section quotes the same band.
/// Dröge & Verstappen (2005), Table II, is the secondary reference for the pressure/friction split
/// only: `C_d = 1.24 = 0.93 + 0.31`, i.e. friction ≈ 25 %.
const ST_REFERENCE: f64 = 0.164;
const CD_REFERENCE_BAND: (f64, f64) = (1.32, 1.36);

/// Read an `f64` case parameter from the environment, falling back to `default`.
fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Read a `usize` case parameter from the environment, falling back to `default`.
fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn main() {
    // Swept parameters (env-overridable). The cylinder sits at ¼ span downstream, mid-channel, so
    // changing the domain keeps it sensibly placed without extra knobs.
    let re_d = env_f64("RE_D", 100.0);
    let cells_per_d = env_usize("CELLS_PER_D", 8);
    let lx_d = env_f64("LX_D", 12.0);
    let ly_d = env_f64("LY_D", 12.0);
    let steps = env_usize("STEPS", 1500);
    let merge_fraction = env_f64("MERGE", 0.25);
    // Advective CFL number `dt = CFL · h / U`. The flow accelerates to ~1.9 U around the cylinder,
    // so the advective limit binds near CFL ≈ 0.45; keep CFL ≤ 0.4 or the march aborts at step 0.
    let cfl = env_f64("CFL", 0.4);
    // Immersed no-slip mode: aperture-resolved (default) or the staircase baseline for the
    // side-by-side validation comparison (`STAIRCASE=1`). Same geometry; only the wall mechanism.
    let staircase = env_usize("STAIRCASE", 0) == 1;
    // Projection CG tolerance. Unset ⇒ the library machine-epsilon default (divergence ~1e-15, but
    // many iterations on a large ill-conditioned cut-cell system); set e.g. `1e-6` to cut iterations
    // dramatically on fine grids (the dominant speed lever) at the cost of a looser divergence floor.
    let cg_tol = std::env::var("CG_TOL")
        .ok()
        .and_then(|s| s.parse::<f64>().ok());

    let diameter = 1.0_f64;
    let radius = 0.5 * diameter;
    let h = diameter / cells_per_d as f64;
    let nx = (lx_d / h).round() as usize;
    let ny = (ly_d / h).round() as usize;
    let nu = U * diameter / re_d;

    // x: inflow (west) / outflow (east); y: far-field slip walls. The geometry, solver, zones,
    // seed and observables are one `CfdConfigBuilder::march` case; the harness owns only its
    // bespoke per-step probe and the reporting, which ride the pipeline's `run_with` hook.
    let center = [lx_d * 0.25, ly_d * 0.5];
    let mesh = Mesh::box_domain([nx, ny])
        .spacing(h)
        .immersed(Body::disk(center, radius).merge_floor(merge_fraction));
    // The registry the cut-cell census reports on: taken from the configured mesh, so it is the
    // same primitive and merge floor the run materializes rather than a hand-built copy.
    let registry = mesh
        .cut_registry()
        .expect("disk intersection")
        .expect("the mesh carries an immersed body");
    let n_solid = registry
        .iter()
        .filter(|(_, c)| c.class().is_solid())
        .count();
    let n_cut = registry.iter().filter(|(_, c)| c.class().is_cut()).count();

    // Advective limit at the inflow speed, scaled by the CFL number.
    let dt = cfl * h / U;

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
    // The projection CG's iteration count grows with the grid, so the default 1000-iteration budget
    // starves the seed/step solves on finer grids. Scale it with the grid (env-overridable).
    let cg_max_iter = env_usize("CG_MAX_ITER", 30 * (nx + ny));
    let ready = CfdConfigBuilder::dec_ns()
        .viscosity(nu)
        .time_step(dt)
        .cg_options(HodgeDecomposeOptions {
            tolerance: cg_tol,
            max_iterations: Some(cg_max_iter),
        })
        // Warm-start the per-stage projection CG from the previous solve's potential. As the flow
        // develops the right-hand side changes little, so CG converges in a handful of iterations.
        .warm_start();
    // Default is the aperture-resolved cut-face no-slip (auto-on with Cut cells); `STAIRCASE=1`
    // flips to the staircase baseline for the comparison.
    let solver_config = if staircase {
        ready.staircase_noslip()
    } else {
        ready
    }
    .build()
    .expect("solver config");

    // Symmetry-breaking initial condition: uniform stream `U` in x plus a single-signed transverse
    // blob one diameter behind the cylinder. The seed projection makes it divergence-free.
    let (xb, yb) = (center[0] + diameter, center[1]);

    // Cycle-mean drag: the reference `C_d` / `C_l` are cycle means, so the force is sampled over the
    // developed (second-half) window and averaged. The pipeline computes the coefficients and their
    // pressure/friction split every step; the harness averages the same developed-window instants.
    let config = CfdConfigBuilder::march::<2, f64>("dec-cylinder")
        .mesh(mesh)
        .solver(solver_config)
        .zones(zones)
        .seed(Seed::UniformXPerturbed {
            speed: U,
            center: [xb, yb, 0.0],
            sigma: PERTURB_SIGMA * diameter,
            amplitude: PERTURB_EPS,
        })
        .march_for(steps)
        .observe(Observe::default().drag(U).drag_split())
        .build()
        .expect("cylinder case config");
    let manifold = config.materialize().expect("case geometry");

    // Wake probe: transverse (y) velocity ~1.5 D downstream of the cylinder, mid-channel. Read as a
    // raw edge cochain entry (the edge-indexed probe `StepView::one_form` exists for), not the
    // interpolated `Observe::probe`, so the Strouhal signal is the one the pinned band describes.
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
        "# isolated cylinder: grid {nx}\u{d7}{ny} ({cells_per_d}/D), domain {lx_d}\u{d7}{ly_d} D, Re_D={re_d}, nu={nu:.3e}, dt={dt:.3e}"
    );
    let noslip_mode = if staircase {
        "staircase"
    } else {
        "aperture-resolved"
    };
    eprintln!(
        "# cut cells: {n_solid} solid, {n_cut} cut; no-slip {noslip_mode}; merge floor {merge_fraction}; CFL {cfl}; cg_tol {cg_tol:?}; cg_max_iter {cg_max_iter}"
    );
    eprintln!(
        "# trigger: transverse seed eps={PERTURB_EPS} sigma={PERTURB_SIGMA} D at ({xb:.1},{yb:.1}) D"
    );
    println!("step,t,max_speed,interior_div,v_probe");

    let mut probe_series: Vec<(f64, f64)> = Vec::with_capacity(steps);
    let report_every = (steps / 200).max(1);
    let drag_every = (steps / 80).max(1);
    let mut last_step = 0usize;

    let report = CfdFlow::march(&config).on(&manifold).run_with(|view| {
        let step = view.step();
        let t = view.time();
        let u = view.one_form();
        let v_probe = u.as_slice()[probe_edge] / h;
        last_step = step;
        probe_series.push((t, v_probe));
        if step % report_every == 0 {
            // The global divergence residual includes the open inlet/outlet (where the boundary
            // flux makes \u{3b4}u nonzero by design); the *interior* divergence is the meaningful check.
            let codiff = view
                .manifold()
                .codifferential_of(u.as_slice(), 1)
                .into_vec();
            let interior_div = view
                .manifold()
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
                step,
                t,
                view.max_speed().expect("max speed"),
                interior_div,
                v_probe,
            );
        }
    });

    // A solver error is a hard failure, not a stopping condition: reporting St and C_d from a
    // truncated series would produce plausible-looking numbers and a success exit code.
    let report = match report {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[FAIL] march diverged after step {last_step}: {e}");
            eprintln!(
                "=== dec_cylinder_verification FAILED: solver error, no results reported. ==="
            );
            std::process::exit(1);
        }
    };

    // The observed series carry the seed at index 0 and step `k` at index `k`, so the developed
    // window is exactly the instants the hand-rolled loop sampled.
    let series = |name: &str| {
        report
            .series(name)
            .unwrap_or_else(|| panic!("the {name} series was observed"))
            .to_vec()
    };
    let (cd_series, cl_series) = (series("drag"), series("lift"));
    let (cdp_series, cdf_series) = (series("drag_pressure"), series("drag_friction"));
    let drag_samples: Vec<[f64; 4]> = (1..=steps)
        .filter(|k| *k > steps / 2 && k % drag_every == 0)
        .map(|k| [cd_series[k], cl_series[k], cdp_series[k], cdf_series[k]])
        .collect();

    let st = report_strouhal(&probe_series, diameter, U);
    let cd = report_drag_mean(&drag_samples);

    if !verify(st, cd, re_d, cells_per_d) {
        std::process::exit(1);
    }
}

/// Self-verification (exit nonzero on break). Gates that the case actually shed and produced a
/// developed-window drag, and — at the reference Reynolds number only — that `St` and `C_d` sit
/// inside their pinned tripwire bands.
///
/// BREAKING CONDITIONS: a march that never sheds leaves `st = None` and fails gate 1; a march that
/// diverges exits before reaching here (see the step loop); a solver change that moves `St` or
/// `C_d` more than ~10 % fails gate 3 or 4.
fn verify(st: Option<f64>, cd: Option<f64>, re_d: f64, cells_per_d: usize) -> bool {
    let mut ok = true;
    println!("\n--- isolated-cylinder gates (Re_D = {re_d}, {cells_per_d} cells/D) ---");

    // 1. The case shed at all — without this the Strouhal is undefined and the run proved nothing.
    match st {
        Some(v) => println!("  [PASS] [tripwire] shedding detected: St = {v:.4}"),
        None => {
            println!("  [FAIL] [tripwire] no shedding detected — St undefined");
            ok = false;
        }
    }

    // 2. The developed window produced drag samples.
    match cd {
        Some(v) => println!("  [PASS] [tripwire] cycle-mean drag measured: C_d = {v:.3}"),
        None => {
            println!("  [FAIL] [tripwire] no developed-window drag samples");
            ok = false;
        }
    }

    // 3-4. Reference-condition bands. Applied only at Re = 100, which is what the published
    // references describe; a Reynolds ladder run reports without gating rather than passing
    // silently against a band that does not describe it.
    if (re_d - REFERENCE_RE_D).abs() > f64::EPSILON {
        println!(
            "  [SKIP] St / C_d bands describe Re = {REFERENCE_RE_D}; not applicable at Re = {re_d}"
        );
        return ok;
    }

    if let Some(v) = st {
        let pass = v > ST_TRIPWIRE.0 && v < ST_TRIPWIRE.1;
        println!(
            "  [{}] [tripwire] St {v:.4} in [{}, {}]  (reference: Williamson {ST_REFERENCE}, \
             measured is {:+.1} % — this grid is below reference quality)",
            if pass { "PASS" } else { "FAIL" },
            ST_TRIPWIRE.0,
            ST_TRIPWIRE.1,
            100.0 * (v - ST_REFERENCE) / ST_REFERENCE,
        );
        ok &= pass;
    }

    if let Some(v) = cd {
        let pass = v > CD_TRIPWIRE.0 && v < CD_TRIPWIRE.1;
        println!(
            "  [{}] [tripwire] C_d {v:.3} in [{}, {}]  (reference band: 2-D laminar consensus \
             {:.2}–{:.2}, measured is {:+.1} % relative to the band top)",
            if pass { "PASS" } else { "FAIL" },
            CD_TRIPWIRE.0,
            CD_TRIPWIRE.1,
            CD_REFERENCE_BAND.0,
            CD_REFERENCE_BAND.1,
            100.0 * (v - CD_REFERENCE_BAND.1) / CD_REFERENCE_BAND.1,
        );
        ok &= pass;
    }

    if ok {
        println!("=== All isolated-cylinder gates passed. ===");
    } else {
        println!("=== Gate REGRESSION in dec_cylinder_verification: see the FAIL lines. ===");
    }
    ok
}

/// Report the cycle-mean drag/lift over the developed-window samples, with the C_d swing.
/// Returns the cycle-mean `C_d`, or `None` when the window produced no samples.
fn report_drag_mean(samples: &[[f64; 4]]) -> Option<f64> {
    if samples.is_empty() {
        eprintln!("# drag: no developed-window samples");
        return None;
    }
    let n = samples.len() as f64;
    let mean = |k: usize| samples.iter().map(|s| s[k]).sum::<f64>() / n;
    let (cd, cl, cd_p, cd_f) = (mean(0), mean(1), mean(2), mean(3));
    let cd_min = samples.iter().map(|s| s[0]).fold(f64::INFINITY, f64::min);
    let cd_max = samples
        .iter()
        .map(|s| s[0])
        .fold(f64::NEG_INFINITY, f64::max);
    eprintln!(
        "# drag (cycle mean over {} samples): C_d ≈ {cd:.3} (pressure {cd_p:.3} + friction {cd_f:.3}), \
         C_l ≈ {cl:.3}, C_d swing [{cd_min:.3}, {cd_max:.3}]  (ref C_d ≈ {:.2}–{:.2}, friction ≈ 25%)",
        samples.len(),
        CD_REFERENCE_BAND.0,
        CD_REFERENCE_BAND.1,
    );
    Some(cd)
}

/// Estimate `St = f·D/U` from the wake probe's mean-crossing rate over the developed (second-half)
/// signal, and compare to the Williamson isolated-cylinder reference at Re = 100.
/// Returns the measured `St`, or `None` when no shedding was detected.
fn report_strouhal(series: &[(f64, f64)], diameter: f64, u_ref: f64) -> Option<f64> {
    if series.len() < 16 {
        eprintln!("# Strouhal: insufficient samples");
        return None;
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
        return None;
    }
    let period = (crossings.last().unwrap() - crossings[0]) / (crossings.len() - 1) as f64;
    let st = (1.0 / period) * diameter / u_ref;
    eprintln!(
        "# shedding: period {period:.4}, St = f·D/U ≈ {st:.4}  (Williamson Re=100 ≈ {ST_REFERENCE})"
    );
    Some(st)
}
