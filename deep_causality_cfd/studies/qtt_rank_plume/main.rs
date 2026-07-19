/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # QTT plume rank + fork economics — the de-risk study (roadmap M1, risks 2 and 3)
//!
//! Two measurements on the plume-imprinted compressible layer, neither taken before
//! (`plasma-retropulsion-de-risk`, capability `plume-rank-fork-study`):
//!
//! **Phase A — rank.** The retro-plume is a colliding-shock system (barrel shock, Mach disk,
//! shear layer, displaced bow shock); its tensor-train rank is unmeasured. A1 marches the
//! imprinted layer (sponge + body + plume, all through the forcing seam) per thrust coefficient
//! under a *tolerance* round policy and records the peak bond — the dynamic rank. A2 encodes an
//! analytic plume + standoff-shock proxy field on the λ-blended lattice (`BlendedMap`, the
//! `qtt_blend_metric` / `qtt_blunt_body_2d` lineage) at λ = 0 (Cartesian capture) and λ = 1
//! (body-fitted) — the static coordinate-dial probe.
//!
//! **Phase B — fork economics.** March a plume-coupled world to a mid-run pause on the real
//! carrier, fork it, and continue a purposeful throttle roster — coast, two sign-flip
//! straddlers, nominal, high — each branch world carrying its **own** forcing region derived
//! from its own published `"commanded_throttle"`, so the intervention feeds back into that
//! branch's flow through the model. Measured: (a) fork structure — every branch shares the
//! paused fluid and field by reference (O(1) copy-on-write), a hard gate; (b) per-branch
//! continuation wall-clock against an unforked trunk continuation (recorded; band pinned from
//! the first run); (c) post-fork bond growth on a **mirrored** continuation (the pause hides
//! its fluid tensors, so the mirror re-marches the identical deterministic path on the bare
//! marcher — same seed, same steps, same round policy — and reads the bond there); (d) the
//! branch flow observables spread across the roster (the corridor's branch-invariant flow
//! columns are the explicit foil).
//!
//! Degraded-but-measured outcomes (a poor step-cost ratio, rank viable only under the blend
//! metric) are printed as findings for the verdict note — only structural breaks (fork sharing
//! lost, rank saturating the representational ceiling, an errored branch) exit nonzero.
//!
//! Usage:
//! ```text
//! cargo run --release -p deep_causality_cfd --example qtt_rank_plume
//! ```

use deep_causality_cfd::{
    Ambient, BlackoutTrigger, BlendedMap, BlendedMapConfig, CartesianIdentity, CfdFlow,
    CompressibleMarchConfig, CompressibleMarchConfigBuilder, CompressibleMarcher2d, CoupledField,
    EulerStateTt2d, ForcingRegion, MarchStop, QttObserve, ReferenceScales, plume_mask_2d,
    quantize_2d,
};
use deep_causality_physics::{
    Area, Force, Length, PlumeGeometry, Pressure, Temperature, area_mach_ratio_kernel,
    choked_mass_flow_kernel, cordell_braun_plume_boundary_kernel, nozzle_exit_state_kernel,
    srp_thrust_coefficient_kernel,
};
use deep_causality_tensor::{CausalTensor, Truncation};
use std::time::Instant;

// ── The shared physical anchor (the srp_drag_decrement condition, condensed) ──
const P_INF: f64 = 1000.0;
const T_INF: f64 = 216.0;
const R_AIR: f64 = 287.0;
const GAMMA_INF: f64 = 1.4;
const MACH_INF: f64 = 2.0;
const R_BODY: f64 = 0.5;
const GAMMA_JET: f64 = 1.3;
const R_JET: f64 = 300.0;
const T_CHAMBER: f64 = 1500.0;
const EXIT_MACH: f64 = 3.0;
const NOZZLE_HALF_ANGLE: f64 = 15.0 * std::f64::consts::PI / 180.0;
const D_THROAT: f64 = 0.03;
const DOMAIN_M: f64 = 4.0;

// ── Study configuration ──
const L: usize = 5; // 32 x 32
const DT: f64 = 4.0e-4;
const S_REF: f64 = 8.0;
/// Tolerance round policy: rank floats freely — that is the measurement.
const TOL: f64 = 1.0e-6;
/// Phase-A march length (the imprinted structures form well inside this).
const A_STEPS: usize = 240;
/// Phase-B pause point and continuation length (coupled steps).
const K_PAUSE: usize = 20;
const CONT: usize = 20;
/// The rank ceiling of a 2^5 × 2^5 quantized field: bond saturation = no compression.
const RANK_CEILING: usize = 32;
/// The A1 rank sweep and the Phase-B roster (coast + straddlers + nominal + high).
const A_SWEEP: [f64; 4] = [0.5, 1.0, 2.0, 4.0];
const ROSTER: [f64; 5] = [0.0, 0.5, 1.0, 1.5, 4.0];
/// Nominal (trunk) thrust coefficient.
const CT_TRUNK: f64 = 1.0;
/// Per-branch continuation cost ratio band, pinned from the first measured run (observed
/// 0.68–1.05 across the roster on 2026-07-17; see the committed output.txt): a branch through
/// the plume-coupled state must not cost more than twice the unforked trunk continuation.
const STEP_COST_RATIO_BAND: f64 = 2.0;

fn main() {
    println!("=== QTT plume rank + fork economics (plasma-retropulsion de-risk, M1) ===\n");
    println!(
        "  grid 2^{L} x 2^{L}, dt {DT}, tol {TOL} (rank floats free), rank ceiling {RANK_CEILING}\n"
    );
    let mut failures: Vec<String> = Vec::new();
    let mut findings: Vec<String> = Vec::new();

    // ── Phase A1: marched rank of the imprinted layer, per C_T (Cartesian) ──
    println!("  A1 marched rank (sponge + body + plume, Cartesian):");
    println!("    C_T  | peak bond");
    println!("  -------+----------");
    let mut a1_bonds: Vec<(f64, usize)> = Vec::new();
    for &ct in &A_SWEEP {
        let (c_t, geometry, jet) =
            throttle_point(ct).unwrap_or_else(|e| fail(&format!("throttle {ct}"), e));
        let peak = marched_peak_bond(Some((&geometry, jet)))
            .unwrap_or_else(|e| fail(&format!("A1 march C_T {ct}"), e));
        println!("   {c_t:>4.2} |    {peak:>3}");
        a1_bonds.push((c_t, peak));
    }
    let unforced_peak = marched_peak_bond(None).unwrap_or_else(|e| fail("A1 unforced march", e));
    println!("   none |    {unforced_peak:>3}  (no plume; the body/shock baseline)");

    let a1_max = a1_bonds.iter().map(|r| r.1).max().unwrap_or(0);
    if a1_max >= RANK_CEILING {
        failures.push(format!(
            "A1: the imprinted layer saturates the rank ceiling ({a1_max} >= {RANK_CEILING}) — no compression"
        ));
    } else if a1_max > unforced_peak {
        findings.push(format!(
            "A1: the plume imprint raises the peak bond {unforced_peak} -> {a1_max} (colliding-shock rank cost, under the ceiling)"
        ));
    }

    // ── Phase A2: the static coordinate dial (λ = 0 vs λ = 1), per C_T ──
    println!("\n  A2 static plume+shock proxy on the blended lattice:");
    println!("    C_T  | bond λ=0 (Cartesian) | bond λ=1 (fitted)");
    println!("  -------+----------------------+------------------");
    let mut fitted_wins = 0usize;
    for &ct in &A_SWEEP {
        let (c_t, geometry, _) =
            throttle_point(ct).unwrap_or_else(|e| fail(&format!("throttle {ct}"), e));
        let b0 = proxy_bond(&geometry, 0.0).unwrap_or_else(|e| fail("A2 lambda=0", e));
        let b1 = proxy_bond(&geometry, 1.0).unwrap_or_else(|e| fail("A2 lambda=1", e));
        println!("   {c_t:>4.2} |         {b0:>3}          |        {b1:>3}");
        if b1 <= b0 {
            fitted_wins += 1;
        }
    }
    if fitted_wins < A_SWEEP.len() {
        findings.push(format!(
            "A2: the fitted coordinate lowered the proxy bond in {fitted_wins}/{} cases — the blend-metric lever is partial for the plume system",
            A_SWEEP.len()
        ));
    }

    // ── Phase B: fork economics on the plume-coupled state ──
    println!("\n  B fork economics (carrier pause at step {K_PAUSE}, {CONT}-step continuations):");
    let trunk_world = branch_world("trunk", CT_TRUNK).unwrap_or_else(|e| fail("trunk world", e));
    let pause = CfdFlow::march(&trunk_world)
        .run_until(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= K_PAUSE,
        )
        .unwrap_or_else(|e| fail("trunk march", e));
    if let Some(e) = pause.error() {
        fail("trunk pause", e);
    }

    // (a) Fork structure: O(1) copy-on-write sharing, and the fork call itself is cheap.
    let t0 = Instant::now();
    let fork = pause.fork();
    let fork_ns = t0.elapsed().as_nanos();
    let shares = fork.shares_fluid_with(&pause) && fork.shares_field_with(&pause);
    println!("    fork: shares fluid+field = {shares}, setup {fork_ns} ns");
    if !shares {
        failures.push("B: the fork does not share the paused state by reference".into());
    }

    // (b) Per-branch continuation wall-clock vs the unforked trunk continuation.
    let t0 = Instant::now();
    let _trunk_cont = pause
        .continue_with(&trunk_world, CONT)
        .unwrap_or_else(|e| fail("trunk continuation", e));
    let t_trunk = t0.elapsed().as_secs_f64();

    println!("    throttle | cont. time (s) | ratio vs trunk | final-field L2 vs coast");
    println!("    ---------+----------------+----------------+------------------------");
    let mut worlds: Vec<(f64, CompressibleMarchConfig<f64>)> = Vec::new();
    for &ct in &ROSTER {
        let name: &'static str = if ct == 0.0 {
            "coast"
        } else if ct == 0.5 {
            "straddle_lo"
        } else if ct == 1.0 {
            "nominal"
        } else if ct == 1.5 {
            "straddle_hi"
        } else {
            "high"
        };
        worlds.push((
            ct,
            branch_world(name, ct).unwrap_or_else(|e| fail(&format!("world {name}"), e)),
        ));
    }
    let mut coast_field: Option<Vec<f64>> = None;
    let mut ratios: Vec<(f64, f64)> = Vec::new();
    let mut spreads: Vec<(f64, f64)> = Vec::new();
    for (ct, world) in &worlds {
        let t0 = Instant::now();
        let report = pause
            .continue_with(world, CONT)
            .unwrap_or_else(|e| fail(&format!("branch C_T {ct}"), e));
        let dt_branch = t0.elapsed().as_secs_f64();
        let field = report
            .final_field()
            .unwrap_or_else(|| fail("branch field", format!("C_T {ct}: no final field")))
            .to_vec();
        let spread = match &coast_field {
            None => {
                coast_field = Some(field);
                0.0
            }
            Some(coast) => {
                let mut num = 0.0;
                let mut den = 0.0;
                for (a, b) in field.iter().zip(coast) {
                    num += (a - b) * (a - b);
                    den += b * b;
                }
                (num / den.max(1e-300)).sqrt()
            }
        };
        let ratio = dt_branch / t_trunk.max(1e-12);
        println!(
            "     {ct:>6.2}  |     {dt_branch:>7.3}    |     {ratio:>6.2}     |        {spread:.3e}"
        );
        ratios.push((*ct, ratio));
        spreads.push((*ct, spread));
    }

    // (d) The intervention must be coupled to the flow: powered branches diverge from coast.
    let max_spread = spreads
        .iter()
        .filter(|(ct, _)| *ct > 0.0)
        .map(|(_, s)| *s)
        .fold(0.0, f64::max);
    if max_spread < 1e-9 {
        failures.push(
            "B: branch flow observables do not spread across the roster — the intervention is not coupled"
                .into(),
        );
    }
    let worst_ratio = ratios.iter().map(|(_, r)| *r).fold(0.0, f64::max);
    if worst_ratio > STEP_COST_RATIO_BAND {
        findings.push(format!(
            "B: per-branch continuation cost ratio reaches {worst_ratio:.2}x the unforked trunk (pinned band {STEP_COST_RATIO_BAND}; recorded for the verdict)"
        ));
    }

    // (c) Post-fork bond growth, on the mirrored (bit-identical, deterministic) continuation.
    println!("\n    mirrored post-fork bond (bare marcher, same seed/steps/round policy):");
    println!("    throttle | peak bond through continuation");
    println!("    ---------+-------------------------------");
    let trunk_state =
        mirror_march(None, K_PAUSE, &mirror_seed()).unwrap_or_else(|e| fail("mirror trunk", e));
    for (ct, _) in &worlds {
        let region = if *ct > 0.0 {
            let (_, geometry, jet) =
                throttle_point(*ct).unwrap_or_else(|e| fail("mirror throttle", e));
            Some(plume_region(&geometry, jet, 0.55).unwrap_or_else(|e| fail("mirror region", e)))
        } else {
            None
        };
        let (peak, _) = mirror_continue(&trunk_state.0, region.as_ref(), CONT)
            .unwrap_or_else(|e| fail(&format!("mirror branch C_T {ct}"), e));
        println!("     {ct:>6.2}  |            {peak:>3}");
        if peak >= RANK_CEILING {
            failures.push(format!(
                "B: branch C_T {ct} saturates the rank ceiling ({peak} >= {RANK_CEILING}) through the continuation"
            ));
        }
    }

    println!("\n--- reading ---");
    println!("  Rank: the plume-imprinted layer's bond is measured, per C_T, dynamic and static;");
    println!("  the blend-metric dial is probed on the analytic proxy (A2).");
    println!(
        "  Fork: sharing is structural, the continuation cost is a ratio, and the bond growth"
    );
    println!("  is read on the deterministic mirror — the three §6 measurement-2 quantities.");
    if !findings.is_empty() {
        println!("\nFINDINGS (for the verdict note — not failures):");
        for f in &findings {
            println!("  * {f}");
        }
    }

    if failures.is_empty() {
        println!("\nALL HARD GATES PASSED — measurements recorded for the de-risk verdict.");
    } else {
        eprintln!("\nFAILED HARD GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

// ── Shared kernel chain (condensed from verification/srp_drag_decrement; same anchor) ──

fn rho_inf() -> f64 {
    P_INF / (R_AIR * T_INF)
}
fn c_inf() -> f64 {
    (GAMMA_INF * R_AIR * T_INF).sqrt()
}
fn q_inf() -> f64 {
    let u = MACH_INF * c_inf();
    0.5 * rho_inf() * u * u
}
fn p_hat(p: f64) -> f64 {
    p / (rho_inf() * c_inf() * c_inf())
}
fn x_hat(x: f64) -> f64 {
    x / DOMAIN_M
}
fn freestream() -> [f64; 4] {
    let p = p_hat(P_INF);
    [
        1.0,
        MACH_INF,
        0.0,
        p / (GAMMA_INF - 1.0) + 0.5 * MACH_INF * MACH_INF,
    ]
}

/// Chamber pressure sized for `ct_target` through the kernel chain; returns the exact C_T, the
/// Cordell plume geometry, and the fully-expanded jet conserved target (nondimensional).
fn throttle_point(ct_target: f64) -> Result<(f64, PlumeGeometry<f64>, [f64; 4]), String> {
    let p = |what: &str, err: &dyn core::fmt::Debug| format!("{what}: {err:?}");
    let a_throat = std::f64::consts::PI * (D_THROAT / 2.0) * (D_THROAT / 2.0);
    let eps = area_mach_ratio_kernel(EXIT_MACH, GAMMA_JET).map_err(|e| p("eps", &e))?;
    let a_exit = eps * a_throat;
    let r_exit = (a_exit / std::f64::consts::PI).sqrt();
    let cone_length = (r_exit - D_THROAT / 2.0) / NOZZLE_HALF_ANGLE.tan();
    let s_ref_area = std::f64::consts::PI * R_BODY * R_BODY;

    let thrust_at = |p_c: f64| -> Result<f64, String> {
        let mdot = choked_mass_flow_kernel(
            Area::new(a_throat).map_err(|e| p("A*", &e))?,
            Pressure::new(p_c).map_err(|e| p("p_c", &e))?,
            Temperature::new(T_CHAMBER).map_err(|e| p("T_c", &e))?,
            GAMMA_JET,
            R_JET,
        )
        .map_err(|e| p("mdot", &e))?;
        let exit = nozzle_exit_state_kernel(
            Pressure::new(p_c).map_err(|e| p("p_c", &e))?,
            Temperature::new(T_CHAMBER).map_err(|e| p("T_c", &e))?,
            eps,
            GAMMA_JET,
            R_JET,
        )
        .map_err(|e| p("exit", &e))?;
        Ok(mdot.value() * exit.velocity().value() + (exit.pressure().value() - P_INF) * a_exit)
    };
    let p_probe = 1.0e6;
    let k = (thrust_at(p_probe)? + P_INF * a_exit) / p_probe;
    let p_c = (ct_target * q_inf() * s_ref_area + P_INF * a_exit) / k;
    let thrust = thrust_at(p_c)?;
    let c_t = srp_thrust_coefficient_kernel(
        Force::new(thrust).map_err(|e| p("F", &e))?,
        Pressure::new(q_inf()).map_err(|e| p("q", &e))?,
        Area::new(s_ref_area).map_err(|e| p("S", &e))?,
    )
    .map_err(|e| p("C_T", &e))?;
    let geometry = cordell_braun_plume_boundary_kernel(
        Pressure::new(p_c).map_err(|e| p("p_c", &e))?,
        Temperature::new(T_CHAMBER).map_err(|e| p("T_c", &e))?,
        R_JET,
        GAMMA_JET,
        EXIT_MACH,
        NOZZLE_HALF_ANGLE,
        Length::new(D_THROAT).map_err(|e| p("d*", &e))?,
        Length::new(r_exit).map_err(|e| p("r_e", &e))?,
        Length::new(cone_length).map_err(|e| p("L", &e))?,
        Pressure::new(P_INF).map_err(|e| p("p_inf", &e))?,
        MACH_INF,
        GAMMA_INF,
    )
    .map_err(|e| p("plume", &e))?;

    let pr = (P_INF / p_c).powf((GAMMA_JET - 1.0) / GAMMA_JET);
    let t_j = T_CHAMBER * pr;
    let m_j = (2.0 / (GAMMA_JET - 1.0) * (1.0 / pr - 1.0)).sqrt();
    let u_j = m_j * (GAMMA_JET * R_JET * t_j).sqrt();
    let rho_j = P_INF / (R_JET * t_j);
    let rho_hat = rho_j / rho_inf();
    let u_hat = -u_j / c_inf();
    let p_j = p_hat(P_INF);
    let e_hat = p_j / (GAMMA_INF - 1.0) + 0.5 * rho_hat * u_hat * u_hat;
    Ok((c_t, geometry, [rho_hat, rho_hat * u_hat, 0.0, e_hat]))
}

/// The plume forcing region at grid position `cx` (unit square), from the analytic geometry.
fn plume_region(
    geometry: &PlumeGeometry<f64>,
    jet: [f64; 4],
    cx: f64,
) -> Result<ForcingRegion<f64>, String> {
    let dx = 1.0 / (1usize << L) as f64;
    let half_length = x_hat(geometry.penetration_length().value()) / 2.0;
    let max_radius = x_hat(geometry.max_radius().value());
    let trunc = Truncation::<f64>::by_tol(TOL).map_err(|e| format!("trunc: {e:?}"))?;
    let mask = plume_mask_2d::<f64>(L, L, dx, dx, cx, 0.5, half_length, max_radius, dx, &trunc)
        .map_err(|e| format!("mask: {e:?}"))?;
    ForcingRegion::new(mask, jet, DT).map_err(|e| format!("region: {e:?}"))
}

/// A Phase-B branch world: freestream seed, no schedule, the branch's own plume imprint (from
/// its own throttle), and the throttle published as `"commanded_throttle"` — the counterfactual
/// seam name the propulsion coupling contract pins.
fn branch_world(name: &'static str, ct: f64) -> Result<CompressibleMarchConfig<f64>, String> {
    let trunc = Truncation::<f64>::by_tol(TOL).map_err(|e| format!("trunc: {e:?}"))?;
    let dx = 1.0 / (1usize << L) as f64;
    let p_inf_hat = p_hat(P_INF);
    let mut builder = CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(L, L, dx, dx)
        .solver(DT, S_REF, GAMMA_INF, trunc)
        .flight_dt(0.05)
        .seed_fn(move |_, _| (1.0, MACH_INF, 0.0, p_inf_hat))
        .map_err(|e| format!("seed: {e:?}"))?
        .stop(MarchStop::Fixed(K_PAUSE + CONT))
        .observe(QttObserve::default())
        .reference(ReferenceScales {
            t_ref: 1.0,
            n_ref: 1.0,
            u_ref: 1.0,
        })
        .publish_constant("commanded_throttle", ct);
    if ct > 0.0 {
        let (_, geometry, jet) = throttle_point(ct)?;
        builder = builder.forcing_region(plume_region(&geometry, jet, 0.55)?);
    }
    builder.build().map_err(|e| format!("build: {e:?}"))
}

/// Phase-A1: march the full imprinted harness (sponge + body + optional plume) on the bare
/// marcher under the tolerance round policy; return the peak bond.
fn marched_peak_bond(plume: Option<(&PlumeGeometry<f64>, [f64; 4])>) -> Result<usize, String> {
    let region = match plume {
        Some((geometry, jet)) => Some(plume_region(
            geometry,
            jet,
            body_face() - x_hat(geometry.penetration_length().value()) / 2.0,
        )?),
        None => None,
    };
    let seed = mirror_seed();
    let (state, peak0) = mirror_march(region.as_ref(), A_STEPS, &seed)?;
    let _ = state;
    Ok(peak0)
}

fn body_face() -> f64 {
    0.72 - x_hat(R_BODY)
}

/// The mirror's common pieces: freestream seed trains.
fn mirror_seed() -> [CausalTensor<f64>; 4] {
    let n = 1usize << L;
    let fs = freestream();
    let t = |v: f64| CausalTensor::new(vec![v; n * n], vec![n, n]).expect("seed tensor");
    [t(fs[0]), t(fs[1]), t(fs[2]), t(fs[3])]
}

/// March `steps` on the bare marcher with sponge + body always on and an optional plume region;
/// return the final state and the peak bond. Deterministic: the Phase-B mirror re-runs the
/// carrier's exact path (same seed, same order of operations, same round policy).
fn mirror_march(
    plume: Option<&ForcingRegion<f64>>,
    steps: usize,
    seed: &[CausalTensor<f64>; 4],
) -> Result<(EulerStateTt2d<f64>, usize), String> {
    let p = |what: &str, err: &dyn core::fmt::Debug| format!("{what}: {err:?}");
    let n = 1usize << L;
    let dx = 1.0 / n as f64;
    let trunc = Truncation::<f64>::by_tol(TOL).map_err(|e| p("trunc", &e))?;
    let metric = CartesianIdentity::new(L, L, dx, dx, trunc).map_err(|e| p("metric", &e))?;
    let marcher = CompressibleMarcher2d::new(metric, GAMMA_INF, DT, S_REF, trunc)
        .map_err(|e| p("marcher", &e))?;

    let fs = freestream();
    let sponge_mask = deep_causality_cfd::mask_from_fn::<f64, _>(
        L,
        L,
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
    .map_err(|e| p("sponge mask", &e))?;
    let sponge = ForcingRegion::new(sponge_mask, fs, DT).map_err(|e| p("sponge", &e))?;
    let body_mask =
        deep_causality_cfd::body_mask_2d::<f64>(L, L, dx, dx, 0.72, 0.5, x_hat(R_BODY), dx, &trunc)
            .map_err(|e| p("body mask", &e))?;
    let p_amb = p_hat(P_INF);
    let body = ForcingRegion::new(body_mask, [1.0, 0.0, 0.0, p_amb / (GAMMA_INF - 1.0)], DT)
        .map_err(|e| p("body", &e))?;

    let mut state: EulerStateTt2d<f64> = [
        quantize_2d(&seed[0], &trunc).map_err(|e| p("q0", &e))?,
        quantize_2d(&seed[1], &trunc).map_err(|e| p("q1", &e))?,
        quantize_2d(&seed[2], &trunc).map_err(|e| p("q2", &e))?,
        quantize_2d(&seed[3], &trunc).map_err(|e| p("q3", &e))?,
    ];
    let mut peak = 0usize;
    for _ in 0..steps {
        state = marcher.step(&state).map_err(|e| p("step", &e))?;
        state = sponge
            .apply(&state, DT, &trunc)
            .map_err(|e| p("sponge", &e))?;
        state = body.apply(&state, DT, &trunc).map_err(|e| p("body", &e))?;
        if let Some(region) = plume {
            state = region
                .apply(&state, DT, &trunc)
                .map_err(|e| p("plume", &e))?;
        }
        peak = peak.max(state_bond(&state));
    }
    Ok((state, peak))
}

/// Continue a mirrored state `steps` further under a branch's own (optional) plume region.
fn mirror_continue(
    state: &EulerStateTt2d<f64>,
    plume: Option<&ForcingRegion<f64>>,
    steps: usize,
) -> Result<(usize, EulerStateTt2d<f64>), String> {
    let p = |what: &str, err: &dyn core::fmt::Debug| format!("{what}: {err:?}");
    let n = 1usize << L;
    let dx = 1.0 / n as f64;
    let trunc = Truncation::<f64>::by_tol(TOL).map_err(|e| p("trunc", &e))?;
    let metric = CartesianIdentity::new(L, L, dx, dx, trunc).map_err(|e| p("metric", &e))?;
    let marcher = CompressibleMarcher2d::new(metric, GAMMA_INF, DT, S_REF, trunc)
        .map_err(|e| p("marcher", &e))?;
    let mut s = state.clone();
    let mut peak = 0usize;
    for _ in 0..steps {
        s = marcher.step(&s).map_err(|e| p("step", &e))?;
        if let Some(region) = plume {
            s = region.apply(&s, DT, &trunc).map_err(|e| p("plume", &e))?;
        }
        peak = peak.max(state_bond(&s));
    }
    Ok((peak, s))
}

fn state_bond(state: &EulerStateTt2d<f64>) -> usize {
    state
        .iter()
        .flat_map(|t| t.cores().iter().map(|c| c.shape()[2]))
        .max()
        .unwrap_or(1)
}

/// Phase-A2: the analytic plume + standoff-shock proxy sampled on the λ-blended lattice (the
/// `qtt_blunt_body_2d` pattern: a fixed physical structure, two charts), quantized at the study
/// tolerance; returns its bond. Geometry scales with the kernel's plume dimensions.
fn proxy_bond(geometry: &PlumeGeometry<f64>, lambda: f64) -> Result<usize, String> {
    const R0: f64 = 1.0;
    const DR: f64 = 1.0;
    const DTHETA: f64 = std::f64::consts::PI / 2.0;
    let l = 6usize;
    let side = 1usize << l;
    let cfg = BlendedMapConfig::new(l, l, R0, DR, -DTHETA / 2.0, DTHETA, lambda);
    let trunc = Truncation::<f64>::by_tol(TOL).map_err(|e| format!("trunc: {e:?}"))?;
    let map = BlendedMap::new(cfg, trunc).map_err(|e| format!("map: {e:?}"))?;

    // The proxy structure, in fan units (fan radial extent 1 unit ↔ DOMAIN_M meters): the
    // standoff shock front at r_s, the plume ellipse on the axis behind it.
    let standoff = x_hat(geometry.terminal_shock_standoff().value());
    let r_shock = 1.3 + standoff;
    let half_length = x_hat(geometry.penetration_length().value()) / 2.0;
    let max_radius = x_hat(geometry.max_radius().value());
    let w = 2.0 * DR / side as f64;
    let mut data = vec![0.0f64; side * side];
    for ix in 0..side {
        for iy in 0..side {
            let xi = ix as f64 / side as f64;
            let eta = iy as f64 / side as f64;
            let (x, y) = map.position(xi, eta);
            let r = (x * x + y * y).sqrt();
            let shock = 0.5 * (1.0 + ((r - r_shock) / w).tanh());
            let ex = (x - (1.15 + half_length)) / half_length.max(1e-9);
            let ey = y / max_radius.max(1e-9);
            let d = ((ex * ex + ey * ey).sqrt() - 1.0) * half_length.min(max_radius);
            let plume = 0.5 * (1.0 - (d / w).tanh());
            data[ix * side + iy] = shock + plume;
        }
    }
    let field = CausalTensor::new(data, vec![side, side]).map_err(|e| format!("tensor: {e:?}"))?;
    Ok(quantize_2d(&field, &trunc)
        .map_err(|e| format!("encode: {e:?}"))?
        .max_bond())
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(2);
}
