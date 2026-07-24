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
//!   with the lift `C_l` and the `C_d` swing. Reference: `C_d(Re=100) ≈ 1.24–1.33`
//!   (Dröge–Verstappen 2005: 1.24 = 0.93 pressure + 0.31 friction; Lehmkuhl et al. 2013 lineage
//!   ≈ 1.33), so friction is ≈ 25 % of `C_d`.
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

use std::collections::BTreeMap;

use deep_causality_cfd::{
    DecNsSolver, Inflow, Outflow, SlipWall, SolenoidalField, force_coefficient,
    pressure_surface_force, viscous_surface_force,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCellRegistry, HodgeDecomposeOptions, LatticeCell,
    LatticeComplex, Manifold, Primitive,
};

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
// reference-grid quality and the measured values sit *outside* the published bands — St 0.1710 vs
// Williamson 0.164 (+4.3 %), C_d 1.345 vs the 1.24–1.33 band (+1.1 % over the top). Gating against
// the published bands at this resolution would fail a correctly-working solver, so these bounds are
// pinned around the measured default and detect regression only. The published values are printed
// next to the measurement so the offset stays visible and is never read as agreement.
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
/// Published references, printed beside the measurement. Williamson (1996) for `St`;
/// Dröge & Verstappen (2005) / Lehmkuhl et al. (2013) for `C_d`.
const ST_REFERENCE: f64 = 0.164;
const CD_REFERENCE_BAND: (f64, f64) = (1.24, 1.33);

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

    // x: inflow (west) / outflow (east); y: far-field slip walls.
    let lattice = LatticeComplex::<2, f64>::new([nx, ny], [false, false]);
    let center = [lx_d * 0.25, ly_d * 0.5];
    let base = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball(center, radius);
    let registry = CutCellRegistry::from_primitive(&lattice, &base, &disk)
        .expect("disk intersection")
        .with_cell_merging(merge_fraction);
    let n_solid = registry
        .iter()
        .filter(|(_, c)| c.class().is_solid())
        .count();
    let n_cut = registry.iter().filter(|(_, c)| c.class().is_cut()).count();
    // The registry is moved into the geometry; keep a copy for the surface-force diagnostics.
    let registry_force = registry.clone();
    let metric = base.with_cut_cells(registry);

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

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
    let solver = DecNsSolver::with_zones(&manifold, nu, dt, zones)
        .expect("solver")
        .with_cg_options(HodgeDecomposeOptions {
            tolerance: cg_tol,
            max_iterations: Some(cg_max_iter),
        })
        // Warm-start the per-stage projection CG from the previous solve's potential. As the flow
        // develops the right-hand side changes little, so CG converges in a handful of iterations.
        .with_warm_start();
    // Default is the aperture-resolved cut-face no-slip (auto-on with Cut cells); `STAIRCASE=1`
    // flips to the staircase baseline for the comparison.
    let solver = if staircase {
        solver.with_staircase_noslip()
    } else {
        solver
    };

    // Symmetry-breaking initial condition: uniform stream `U` in x plus a single-signed transverse
    // blob one diameter behind the cylinder. The seed projection makes it divergence-free.
    let n0 = manifold.complex().num_cells(0);
    let mut vv = vec![0.0_f64; 2 * n0];
    let (xb, yb) = (center[0] + 1.0, center[1]);
    let two_sigma_sq = 2.0 * PERTURB_SIGMA * PERTURB_SIGMA;
    for (i, v) in manifold.complex().iter_cells(0).enumerate() {
        let p = v.position();
        let x_d = p[0] as f64 * h / diameter;
        let y_d = p[1] as f64 * h / diameter;
        let r2 = (x_d - xb).powi(2) + (y_d - yb).powi(2);
        vv[2 * i] = U;
        vv[2 * i + 1] = PERTURB_EPS * U * (-r2 / two_sigma_sq).exp();
    }
    let seed = CausalTensor::new(vv, vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&seed).expect("seed");

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
        "# isolated cylinder: grid {nx}×{ny} ({cells_per_d}/D), domain {lx_d}×{ly_d} D, Re_D={re_d}, nu={nu:.3e}, dt={dt:.3e}"
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
    // Cycle-mean drag: sample the force over the developed (second-half) window and average, since
    // the reference C_d / C_l are cycle means, not a single instant.
    let drag_every = (steps / 80).max(1);
    let mut drag_samples: Vec<[f64; 4]> = Vec::new();
    for step in 0..steps {
        // A solver error is a hard failure, not a stopping condition. The previous `break` fell
        // through to the reporting path, which then computed St and C_d from the *truncated*
        // series and returned 0 — so a diverged march produced plausible-looking numbers and a
        // success exit code, contradicting the suite convention in verification/README.md.
        let out = match solver.step(&state) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("[FAIL] march diverged at step {step}: {e}");
                eprintln!(
                    "=== dec_cylinder_verification FAILED: solver error, no results reported. ==="
                );
                std::process::exit(1);
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
        if step + 1 > steps / 2
            && (step + 1) % drag_every == 0
            && let Some(s) = instantaneous_drag(
                &solver,
                &manifold,
                &registry_force,
                out.state(),
                nu,
                U,
                diameter,
            )
        {
            drag_samples.push(s);
        }
        state = out.into_state();
    }

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
            "  [{}] [tripwire] C_d {v:.3} in [{}, {}]  (reference band: Dröge–Verstappen / \
             Lehmkuhl {:.2}–{:.2}, measured is {:+.1} % relative to the band top)",
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

/// Instantaneous drag/lift coefficients at one state: `[C_d, C_l, C_d_pressure, C_d_friction]`.
/// The pressure force comes from the recovered static pressure, the friction from the viscous
/// surface traction. Returns `None` if a diagnostic solve fails. One CG solve (pressure) per call.
fn instantaneous_drag(
    solver: &DecNsSolver<'_, 2, f64>,
    manifold: &Manifold<LatticeComplex<2, f64>, f64>,
    registry: &CutCellRegistry<2, f64>,
    state: &SolenoidalField<f64>,
    nu: f64,
    u_ref: f64,
    diameter: f64,
) -> Option<[f64; 4]> {
    // Static pressure 0-form at this state (one CG solve), keyed by vertex position.
    let (_bernoulli, static_p) = match solver.pressure_diagnostic(state) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("# drag: pressure diagnostic failed: {e}");
            return None;
        }
    };
    let p_vert = static_p.as_tensor().as_slice();
    let vertex_index: BTreeMap<[usize; 2], usize> = manifold
        .complex()
        .iter_cells(0)
        .enumerate()
        .map(|(i, v)| (*v.position(), i))
        .collect();

    // Per-cell pressure = mean of the cell's corner vertices (the `iter_cells(2)` / CellId order).
    let cell_pressure: Vec<f64> = manifold
        .complex()
        .iter_cells(2)
        .map(|cell: LatticeCell<2>| {
            let corners = cell.vertices();
            let sum: f64 = corners
                .iter()
                .filter_map(|pos| vertex_index.get(pos).map(|&i| p_vert[i]))
                .sum();
            sum / corners.len() as f64
        })
        .collect();

    let f_pressure = pressure_surface_force(registry, |id| cell_pressure[id]);
    let f_viscous = match viscous_surface_force(manifold, registry, state.as_one_form(), nu) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("# drag: viscous force failed: {e}");
            return None;
        }
    };
    let f_total = [f_pressure[0] + f_viscous[0], f_pressure[1] + f_viscous[1]];

    let cd = force_coefficient(f_total[0], u_ref, diameter);
    let cl = force_coefficient(f_total[1], u_ref, diameter);
    let cd_p = force_coefficient(f_pressure[0], u_ref, diameter);
    let cd_f = force_coefficient(f_viscous[0], u_ref, diameter);
    Some([cd, cl, cd_p, cd_f])
}
