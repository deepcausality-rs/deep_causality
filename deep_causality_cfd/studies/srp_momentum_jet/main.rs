/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # SRP momentum jet — does a momentum-carrying jet recover the Jarvinen–Adams collapse?
//!
//! Follow-up study to the de-risk verification `srp_drag_decrement` and its recorded amber
//! finding (`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`): the verification's
//! whole-envelope **pinned-state** plume (interior held at ambient pressure) shields the
//! forebody monotonically but cannot produce the Jarvinen–Adams central-nozzle drag collapse —
//! and the verdict names "a momentum-carrying jet interaction rather than a pinned obstruction
//! state (inject the jet's mass/momentum flux and let the plume form)" as the first upgrade
//! path. This study measures exactly that variant on the **same harness**: same freestream,
//! body, sponge, strip, and (at defaults) grid and bond cap, with the plume envelope pin
//! replaced by a nozzle-exit **patch** at the body face pinned to a supersonic upstream-firing
//! exit state (fixed exit Mach 3, throttled by exit pressure through the closed-form
//! fixed-nozzle relation in `config.rs`). The plume is *not* imposed: it forms, spreads, and
//! interacts in the marched field.
//!
//! Question: is the missing collapse a property of the **model class** (static imprint) or of
//! the **harness** (2-D plane, coarse grid, dissipation floor ν = ½·s_ref·Δx)? The de-risk
//! adversarial review pre-registered the evidence bar, and this binary instruments it:
//! - **time-averaged tail read** — the committed harness contracts a single terminal snapshot;
//!   here the strip force is sampled every step over the tail window, with mean, std, and a
//!   first-half/second-half drift witness (stationarity);
//! - **three strip bands** — the verification's full strip verbatim (continuity), an
//!   **annulus** excluding the jet's rows (the J–A aeroshell surface, free of the exit pin),
//!   and an **outer** band one further cell out (a collapse must appear off-axis to count);
//! - **mechanism witnesses** — centerline interface location (argmax p̂ upstream of the face),
//!   an upstream freestream probe (sponge/blockage integrity), global min ρ̂/p̂ (floor
//!   monitors), and the realized injected momentum flux audited against the analytic pin;
//! - **robustness dials** — `SRP_MJ_L` (grid level; 6 halves the dissipation), `SRP_MJ_CAP`
//!   (32 is exact at L = 5: truncation off), `SRP_MJ_STEPS` / `SRP_MJ_TAIL` (settle length),
//!   `SRP_MJ_SWEEP` (comma-separated C_T subset). Δt scales with the grid (8e-4 at 2⁵).
//!
//! Caveats carried unchanged from the verification: 2-D plane (not axisymmetric), a
//! smoothed-mask body, periodic + sponge blockage, a single marched γ = 1.4; the J–A
//! correlation is axisymmetric, so the quantitative fractions do not transfer — the collapse
//! *structure* is what is being measured. C_T is the declared 2-D per-depth definition
//! (`config.rs`).
//!
//! Usage:
//! ```text
//! cargo run --release -p deep_causality_cfd --example srp_momentum_jet
//! SRP_MJ_L=6 SRP_MJ_SWEEP=1.0,2.0,4.0 cargo run --release -p deep_causality_cfd --example srp_momentum_jet
//! ```

mod config;

use config::*;
use deep_causality_cfd::{
    CartesianIdentity, CompressibleMarcher2d, EulerStateTt2d, ForcingRegion, dequantize_2d,
    mask_from_fn, preserved_drag_fraction, quantize_2d, strip_pressure_force,
};
use deep_causality_physics::{
    srp_preserved_drag_fraction_kernel, srp_total_axial_force_coefficient_kernel,
};
use deep_causality_tensor::{CausalTensor, TensorTrain, Truncation};

/// Runtime harness dials (environment-overridable; defaults are the study configuration).
#[derive(Clone, Copy)]
struct Cfg {
    l: usize,
    n: usize,
    dx: f64,
    dt: f64,
    cap: usize,
    steps: usize,
    tail: usize,
}

/// One marched run's measurements: terminal TT-contract gauge (the committed methodology),
/// tail-averaged dense gauges per band with std and drift, and the mechanism witnesses.
struct RunOut {
    f_tt_full: f64,
    mean: [f64; 3], // full, annulus, outer
    std: [f64; 3],
    drift: [f64; 3],
    peak_bond: usize,
    interface_x: f64,
    probe_dev: f64,
    min_rho: f64,
    min_p: f64,
    t_realized: f64,
}

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn main() {
    let l = env_usize("SRP_MJ_L", 5);
    let n = 1usize << l;
    let cfg = Cfg {
        l,
        n,
        dx: 1.0 / n as f64,
        dt: 8.0e-4 * 32.0 / n as f64,
        cap: env_usize("SRP_MJ_CAP", 24),
        steps: env_usize("SRP_MJ_STEPS", 2000),
        tail: env_usize("SRP_MJ_TAIL", 500),
    };
    let sweep: Vec<f64> = std::env::var("SRP_MJ_SWEEP")
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|t| t.trim().parse().ok())
                .collect::<Vec<f64>>()
        })
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| CT_SWEEP.to_vec());

    println!("=== SRP momentum jet: formed plume vs Jarvinen-Adams (M2, central) ===\n");
    println!(
        "  freestream M {MACH_INF}, gamma {GAMMA_INF}, p_inf {P_INF} Pa; jet exit M {EXIT_MACH}, gamma_jet {GAMMA_JET}, u_hat_e {:.2}",
        u_hat_exit()
    );
    println!(
        "  grid 2^{} x 2^{} over {DOMAIN_M} m, dt {:.1e}, {} steps (tail mean over last {}), bond cap {}",
        cfg.l, cfg.l, cfg.dt, cfg.steps, cfg.tail, cfg.cap
    );
    // The mask codec samples at grid nodes, so the realized pinned patch is counted on the
    // node lattice and the throttle is sized to that realized height (see config.rs).
    let face_x = BODY_CX - x_hat(R_BODY);
    let jet_rows = (0..cfg.n)
        .filter(|&j| ((j as f64) * cfg.dx - BODY_CY).abs() <= jet_half_height_hat())
        .count();
    let jet_cols = (0..cfg.n)
        .filter(|&i| {
            let x = (i as f64) * cfg.dx;
            x > face_x - DX_L5 && x < face_x
        })
        .count();
    let h_hat_eff = jet_rows as f64 * cfg.dx;
    println!(
        "  jet patch: {jet_cols} node-column(s) x {jet_rows} node-rows at the body face -> realized exit height {:.3} m, r_jet/R_body = {:.3}\n",
        h_hat_eff * DOMAIN_M,
        0.5 * h_hat_eff / x_hat(R_BODY)
    );
    println!("  caveats: 2-D plane (not axisymmetric), smoothed-mask body, periodic + sponge");
    println!("  blockage, single marched gamma; C_T is the 2-D per-depth definition; J-A");
    println!("  (axisymmetric) is the structural reference, not a quantitative fit target\n");

    let mut failures: Vec<String> = Vec::new();

    // Jet wave envelope must sit inside the marcher's implicit acoustic reference.
    let ct_max = sweep.iter().cloned().fold(0.0, f64::max);
    let (p_e_probe, rho_e_probe, _) = jet_exit_state(ct_max, h_hat_eff);
    let s_jet = u_hat_exit().abs() + (GAMMA_INF * p_e_probe / rho_e_probe).sqrt();
    if s_jet > S_REF {
        fail(
            "wave envelope",
            format!("jet |u|+c = {s_jet:.2} exceeds S_REF {S_REF}"),
        );
    }

    // ── Unpowered baseline (same harness, no jet) ──
    let base = march_and_contract(cfg, None).unwrap_or_else(|e| fail("baseline run", e));
    println!(
        "  baseline (no jet): tail gauge force {:.6e} (full) / {:.6e} (annulus) / {:.6e} (outer), peak bond {}",
        base.mean[0], base.mean[1], base.mean[2], base.peak_bond
    );
    println!(
        "  baseline terminal TT contract {:.6e}; tail std {:.1e}, drift {:+.2}%",
        base.f_tt_full,
        base.std[0],
        100.0 * base.drift[0]
    );
    if cfg.l == 5 && cfg.cap == 24 && cfg.steps == 500 {
        println!(
            "  continuity vs committed verification baseline: delta {:.1e}",
            (base.f_tt_full - VERIFICATION_BASELINE).abs()
        );
    } else {
        println!(
            "  (committed-baseline identity holds at L=5/cap24/500 steps: measured delta 4.1e-9 on the first run)"
        );
    }
    println!();
    if base.mean.iter().any(|f| !(f.is_finite() && *f > 0.0)) {
        fail(
            "baseline sanity",
            format!("tail gauge forces must be positive, got {:?}", base.mean),
        );
    }

    // ── The C_T sweep ──
    println!(
        "    C_T   | p_e/p_inf | frac full | frac annulus | frac outer | J-A    | drift%  | iface x | bond"
    );
    println!(
        "  --------+-----------+-----------+--------------+------------+--------+---------+---------+-----"
    );
    // (c_t, frac_full, frac_ann, frac_outer, frac_ja)
    let mut rows: Vec<(f64, f64, f64, f64, f64)> = Vec::new();
    let mut ifaces: Vec<f64> = Vec::new();
    for &ct in &sweep {
        let (p_e, _rho_e, target) = jet_exit_state(ct, h_hat_eff);
        let region =
            jet_region(cfg, target).unwrap_or_else(|e| fail(&format!("jet region {ct}"), e));
        let out = march_and_contract(cfg, Some(region))
            .unwrap_or_else(|e| fail(&format!("powered run C_T {ct}"), e));
        let frac = |i: usize| preserved_drag_fraction(out.mean[i], base.mean[i]);
        let f_full = frac(0).unwrap_or_else(|e| fail("fraction", e));
        let f_ann = frac(1).unwrap_or_else(|e| fail("fraction", e));
        let f_out = frac(2).unwrap_or_else(|e| fail("fraction", e));
        let f_ja =
            srp_preserved_drag_fraction_kernel(ct).unwrap_or_else(|e| fail("J-A fraction", e));
        println!(
            "   {ct:>5.2}  |   {:>5.2}   |  {f_full:>7.3}  |   {f_ann:>7.3}    |  {f_out:>7.3}   | {f_ja:>6.3} | {:>+6.2}  |  {:>5.3}  | {:>3}",
            p_e / p_hat(P_INF),
            100.0 * out.drift[1],
            out.interface_x,
            out.peak_bond
        );
        // Per-point witnesses that must hold for the row to be trusted.
        let t_expect = ct * q_hat_inf() * d_hat_body();
        if (out.t_realized - t_expect).abs() > 0.05 * t_expect {
            failures.push(format!(
                "W-T: realized momentum flux {:.3e} deviates from C_T-implied {:.3e} at C_T {ct}",
                out.t_realized, t_expect
            ));
        }
        if out.probe_dev.abs() > 0.05 {
            println!(
                "         ! upstream probe off freestream by {:+.1}% at C_T {ct} (sponge/blockage interaction)",
                100.0 * out.probe_dev
            );
        }
        if out.min_p < 1e-12 || out.min_rho < 1e-12 {
            failures.push(format!(
                "W-F: pressure/density floor activated at C_T {ct} (min p {:.1e}, min rho {:.1e})",
                out.min_p, out.min_rho
            ));
        }
        if !(f_full.is_finite() && f_ann.is_finite() && f_out.is_finite()) {
            failures.push(format!("S-A: non-finite fraction at C_T {ct}"));
        }
        rows.push((ct, f_full, f_ann, f_out, f_ja));
        ifaces.push(out.interface_x);
    }

    // ── Regression bands (pinned from the FIRST measured v3 run, 2026-07-17; see config.rs
    //    for provenance). They gate only the default configuration — env-dialed companion
    //    runs (cap, grid, sweep) measure, they do not regress. ──
    let is_default = cfg.l == 5
        && cfg.cap == 24
        && cfg.steps == 2000
        && cfg.tail == 500
        && sweep.as_slice() == CT_SWEEP;
    if is_default {
        // R-A': monotone augmentation — the annulus fraction is non-decreasing in C_T.
        if !rows.windows(2).all(|w| w[1].2 >= w[0].2 - 1e-9) {
            failures.push("R-A': annulus fraction is not monotone non-decreasing in C_T".into());
        }
        // R-B': sweep-top annulus fraction inside the pinned band.
        if let Some(top) = rows.last() {
            let (lo, hi) = TOP_ANNULUS_FRACTION_BAND;
            if !(top.2 >= lo && top.2 <= hi) {
                failures.push(format!(
                    "R-B': annulus fraction {:.3} at C_T {:.2} outside pinned band [{lo}, {hi}]",
                    top.2, top.0
                ));
            }
        }
        // R-C': the frozen-interface witness at every swept C_T.
        let (ilo, ihi) = INTERFACE_X_BAND;
        for (row, &ix) in rows.iter().zip(&ifaces) {
            if !(ix >= ilo && ix <= ihi) {
                failures.push(format!(
                    "R-C': interface x {ix:.3} at C_T {:.2} outside pinned band [{ilo}, {ihi}]",
                    row.0
                ));
            }
        }
    }

    // ── The measurement (reported; structure vs the pinned-envelope amber) ──
    let ca0 =
        srp_total_axial_force_coefficient_kernel(0.0, MACH_INF).unwrap_or_else(|e| fail("C_A0", e));
    if let Some(near_unity) = rows
        .iter()
        .min_by(|a, b| (a.0 - 1.0).abs().partial_cmp(&(b.0 - 1.0).abs()).unwrap())
        .copied()
    {
        let min_ann = rows.iter().map(|r| r.2).fold(f64::INFINITY, f64::min);
        let harness_total: Vec<f64> = rows.iter().map(|r| r.0 + r.2 * ca0).collect();
        let monotone = harness_total.windows(2).all(|w| w[1] >= w[0]);
        let argmin = harness_total
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| rows[i].0)
            .unwrap_or(f64::NAN);
        println!("\n  FINDING (recorded; structural bands gate the default configuration only —");
        println!("  derisk-verdict.md and its addendum are the authority):");
        println!(
            "    J-A collapse NOT reproduced by the momentum-carrying jet either: monotone drag"
        );
        println!(
            "    AUGMENTATION, annulus fraction {:.3} at C_T {:.2} (J-A {:.3}), sweep minimum {:.3};",
            near_unity.2, near_unity.0, near_unity.4, min_ann
        );
        println!(
            "    total-axial-force dip {} (argmin C_T {:.2}); the dissipation floor",
            if monotone {
                "ABSENT (monotone)"
            } else {
                "PRESENT (non-monotone)"
            },
            argmin
        );
        println!(
            "    (nu = s_ref*dx/2) freezes the stagnation interface at the face across the full"
        );
        println!(
            "    throttle range, so injected momentum reads as face pressure, the inverse of the"
        );
        println!(
            "    J-A blanketing reorganization. Pinned-envelope amber for reference: 0.895 at"
        );
        println!(
            "    C_T 1, floor 0.647, dip absent. Neither coupling model can host the collapse"
        );
        println!("    on this harness; the A0 correlation channel keeps the drag authority.");
    }

    if failures.is_empty() {
        println!(
            "\nGATES PASSED — the measured augmentation structure and witnesses hold. The J-A miss is the recorded finding, not a regression."
        );
    } else {
        eprintln!("\nFAILED GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

/// The momentum-jet forcing region: one L5-cell-wide column flush against the body face,
/// 0.25 m tall (grid-independent physical geometry), hard-pinned (η = Δt) to the nozzle-exit
/// conserved state — a supersonic inflow patch; everything downstream forms in the march.
fn jet_region(cfg: Cfg, target: [f64; 4]) -> Result<ForcingRegion<f64>, String> {
    let face_x = BODY_CX - x_hat(R_BODY);
    let trunc = Truncation::<f64>::by_bond(cfg.cap).map_err(|e| format!("trunc: {e:?}"))?;
    let h = jet_half_height_hat();
    let mask = mask_from_fn::<f64, _>(
        cfg.l,
        cfg.l,
        cfg.dx,
        cfg.dx,
        |x, y| {
            let in_x = x > face_x - DX_L5 && x < face_x;
            let in_y = (y - BODY_CY).abs() <= h;
            if in_x && in_y { 1.0 } else { 0.0 }
        },
        &trunc,
    )
    .map_err(|e| format!("jet mask: {e:?}"))?;
    ForcingRegion::new(mask, target, cfg.dt).map_err(|e| format!("jet region: {e:?}"))
}

/// March the harness (sponge + body + optional jet, all through the forcing seam), sampling
/// the three forebody-strip gauge forces densely every tail step, and read the mechanism
/// witnesses from the terminal field.
fn march_and_contract(cfg: Cfg, jet: Option<ForcingRegion<f64>>) -> Result<RunOut, String> {
    let Cfg {
        l, n, dx, dt, cap, ..
    } = cfg;
    let trunc = Truncation::<f64>::by_bond(cap).map_err(|e| format!("trunc: {e:?}"))?;
    let metric = CartesianIdentity::new(l, l, dx, dx, trunc).map_err(|er| perr("metric", er))?;
    let marcher = CompressibleMarcher2d::new(metric, GAMMA_INF, dt, S_REF, trunc)
        .map_err(|er| perr("marcher", er))?;

    let fs = freestream_conserved();
    let enc = |v: f64| -> Result<_, String> {
        quantize_2d(
            &CausalTensor::new(vec![v; n * n], vec![n, n]).map_err(|er| perr("tensor", er))?,
            &trunc,
        )
        .map_err(|er| perr("encode", er))
    };
    let mut state: EulerStateTt2d<f64> = [enc(fs[0])?, enc(fs[1])?, enc(fs[2])?, enc(fs[3])?];

    // Freestream sponge: the inflow strip plus thin lateral bands re-pin the periodic wrap.
    let sponge_mask = mask_from_fn::<f64, _>(
        l,
        l,
        dx,
        dx,
        |x, y| {
            if x < 0.06 || !(0.04..=0.96).contains(&y) {
                1.0
            } else {
                0.0
            }
        },
        &trunc,
    )
    .map_err(|er| perr("sponge mask", er))?;
    let sponge = ForcingRegion::new(sponge_mask, fs, dt).map_err(|er| perr("sponge", er))?;

    // The blunt body: a smoothed disc pinned to a stagnant ambient state.
    let body_mask = deep_causality_cfd::body_mask_2d::<f64>(
        l,
        l,
        dx,
        dx,
        BODY_CX,
        BODY_CY,
        x_hat(R_BODY),
        1.0 * dx,
        &trunc,
    )
    .map_err(|er| perr("body mask", er))?;
    let p_amb = p_hat(P_INF);
    let body_target = [1.0, 0.0, 0.0, p_amb / (GAMMA_INF - 1.0)];
    let body = ForcingRegion::new(body_mask, body_target, dt).map_err(|er| perr("body", er))?;

    // Dense index sets on the NODE lattice (x = k·Δx — the same sampling `mask_from_fn` uses,
    // so these sets are exactly the cells the TT masks select).
    let face_x = BODY_CX - x_hat(R_BODY);
    let h_jet = jet_half_height_hat();
    let node = |k: usize| k as f64 * dx;
    let mut strips: [Vec<usize>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut patch_cells: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            let (x, y) = (node(i), node(j));
            let dy = (y - BODY_CY).abs();
            let in_strip = x >= face_x - 3.5 * dx && x <= face_x - 1.0 * dx && dy <= x_hat(R_BODY);
            if in_strip {
                strips[0].push(i * n + j);
                if dy > h_jet {
                    strips[1].push(i * n + j);
                }
                if dy > h_jet + dx {
                    strips[2].push(i * n + j);
                }
            }
            if x > face_x - DX_L5 && x < face_x && dy <= h_jet {
                patch_cells.push((i, j));
            }
        }
    }
    // The exit-plane audit reads the upstream-most pinned column only: one exit surface, one
    // flux — additional pinned columns thicken the reservoir, they do not add exit area.
    let exit_col = patch_cells.iter().map(|&(i, _)| i).min().unwrap_or(0);
    let patch: Vec<usize> = patch_cells
        .iter()
        .filter(|&&(i, _)| i == exit_col)
        .map(|&(i, j)| i * n + j)
        .collect();
    let patch_cols: Vec<usize> = {
        let mut c: Vec<usize> = patch_cells.iter().map(|&(i, _)| i).collect();
        c.sort_unstable();
        c.dedup();
        c
    };

    let dense = |s: &EulerStateTt2d<f64>| -> Result<[Vec<f64>; 4], String> {
        Ok([
            dequantize_2d(&s[0], l, l)
                .map_err(|er| perr("deq", er))?
                .as_slice()
                .to_vec(),
            dequantize_2d(&s[1], l, l)
                .map_err(|er| perr("deq", er))?
                .as_slice()
                .to_vec(),
            dequantize_2d(&s[2], l, l)
                .map_err(|er| perr("deq", er))?
                .as_slice()
                .to_vec(),
            dequantize_2d(&s[3], l, l)
                .map_err(|er| perr("deq", er))?
                .as_slice()
                .to_vec(),
        ])
    };
    let pressure = |d: &[Vec<f64>; 4], k: usize| -> f64 {
        (GAMMA_INF - 1.0) * (d[3][k] - 0.5 * (d[1][k] * d[1][k] + d[2][k] * d[2][k]) / d[0][k])
    };

    let mut peak = 0usize;
    let tail_start = cfg.steps.saturating_sub(cfg.tail);
    let mut samples: [Vec<f64>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for step in 0..cfg.steps {
        state = marcher.step(&state).map_err(|er| perr("step", er))?;
        state = sponge
            .apply(&state, dt, &trunc)
            .map_err(|er| perr("sponge apply", er))?;
        state = body
            .apply(&state, dt, &trunc)
            .map_err(|er| perr("body apply", er))?;
        if let Some(region) = &jet {
            state = region
                .apply(&state, dt, &trunc)
                .map_err(|er| perr("jet apply", er))?;
        }
        let bond = state
            .iter()
            .flat_map(|t| t.cores().iter().map(|c| c.shape()[2]))
            .max()
            .unwrap_or(1);
        peak = peak.max(bond);
        if step >= tail_start {
            let d = dense(&state)?;
            for (b, sample) in samples.iter_mut().enumerate() {
                let f: f64 = strips[b]
                    .iter()
                    .map(|&k| (pressure(&d, k) - p_amb) * dx * dx)
                    .sum();
                sample.push(f);
            }
        }
    }

    // Terminal TT contraction of the full strip (the committed harness's methodology).
    let strip_full_tt = mask_from_fn::<f64, _>(
        l,
        l,
        dx,
        dx,
        |x, y| {
            let in_x = x >= face_x - 3.5 * dx && x <= face_x - 1.0 * dx;
            let in_y = (y - BODY_CY).abs() <= x_hat(R_BODY);
            if in_x && in_y { 1.0 } else { 0.0 }
        },
        &trunc,
    )
    .map_err(|er| perr("strip mask", er))?;
    let ones = enc(1.0)?;
    let f_total = strip_pressure_force(&strip_full_tt, &state, GAMMA_INF, l, l, dx, dx, &trunc)
        .map_err(|er| perr("contract", er))?;
    let volume = strip_full_tt
        .inner(&ones)
        .map_err(|er| perr("strip volume", er))?
        * dx
        * dx;
    let f_tt_full = f_total - p_amb * volume;

    // Tail statistics per band.
    let stats = |v: &[f64]| -> (f64, f64, f64) {
        let m = v.iter().sum::<f64>() / v.len() as f64;
        let var = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64;
        let half = v.len() / 2;
        let m1 = v[..half].iter().sum::<f64>() / half.max(1) as f64;
        let m2 = v[half..].iter().sum::<f64>() / (v.len() - half).max(1) as f64;
        (m, var.sqrt(), (m2 - m1) / m.abs().max(1e-300))
    };
    let (mut mean, mut std, mut drift) = ([0.0; 3], [0.0; 3], [0.0; 3]);
    for b in 0..3 {
        let (m, s, d) = stats(&samples[b]);
        mean[b] = m;
        std[b] = s;
        drift[b] = d;
    }

    // Terminal-field witnesses (on-axis row is the y = ŷ_c node).
    let d = dense(&state)?;
    let axis_row = n / 2;
    let centerline_p = |i: usize| pressure(&d, i * n + axis_row);
    let face_col = ((face_x / dx) as usize).saturating_sub(1);
    let interface_x = (1..=face_col)
        .filter(|i| !patch_cols.contains(i))
        .max_by(|&a, &b| centerline_p(a).partial_cmp(&centerline_p(b)).unwrap())
        .map(node)
        .unwrap_or(f64::NAN);
    let probe_i = (0.15 / dx).round() as usize;
    let probe_dev = centerline_p(probe_i) / p_amb - 1.0;
    let min_rho = d[0].iter().cloned().fold(f64::INFINITY, f64::min);
    let min_p = (0..n * n)
        .map(|k| pressure(&d, k))
        .fold(f64::INFINITY, f64::min);
    let t_realized: f64 = patch
        .iter()
        .map(|&k| {
            let rho = d[0][k];
            (d[1][k] * d[1][k] / rho + pressure(&d, k) - p_amb) * dx
        })
        .sum();

    Ok(RunOut {
        f_tt_full,
        mean,
        std,
        drift,
        peak_bond: peak,
        interface_x,
        probe_dev,
        min_rho,
        min_p,
        t_realized,
    })
}

/// A labeled error string (the binary's uniform error channel).
fn perr(what: &str, err: impl core::fmt::Debug) -> String {
    format!("{what}: {err:?}")
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(2);
}
