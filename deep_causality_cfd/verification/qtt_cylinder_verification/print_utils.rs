/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Measurement + display + self-verification for the QTT immersed-cylinder run. `main` runs the CfdFlow
//! march at each bond cap and hands the owned reports here; this layer measures the no-slip interior and
//! the accuracy-vs-bond drag convergence, renders the table, and gates the invariants (exit nonzero on
//! break).

use crate::FloatType;
use crate::config;
use deep_causality_cfd::{EvidenceClass, LadderOutcome, Report, dequantize_2d};

/// Pinned no-slip floor: interior speed must fall below this fraction of the free-stream.
///
/// Evidence class: **tripwire**. Note it is invariant across the whole smoothing ladder: at the
/// superseded `L = 5` configuration the interior max|u| stayed 4.19e-2–4.51e-2 while `C_d` moved
/// 6.1× (AUDIT-REPORT §5b). So it constrains the wall treatment and says nothing about the reported
/// drag. That is why the ladders below exist.
const NO_SLIP_FLOOR: f64 = 0.15;
/// Pinned bond-convergence bound on the relative change between the two finest bond caps.
///
/// Tightened from `0.10`. The successive change at the two finest caps measured `7.9e-13` relative
/// (bond 16 vs 24, at the superseded `L = 5` configuration), so the old bound sat eleven orders of
/// magnitude above the phenomenon it gated and would have passed a solver that had not saturated in
/// bond at all. `1e-6` is still ~6 orders above that measurement, which absorbs cross-platform
/// floating-point differences while actually constraining saturation. The shipped `L = 8` ladder
/// (`[24, 48]`) has not been measured against this bound; that acceptance run is the pending offline
/// item recorded in `verification/README.md`.
const CONVERGENCE_BOUND: f64 = 1.0e-6;
/// Pinned blow-up guard on the drag coefficient. A **wide non-divergence bracket**, not a physicality
/// bound.
///
/// What it catches: a negative, NaN, or diverged `C_d`. What it does not catch: any regression that
/// leaves the drag inside `(0, 100)`. The gap is wide on purpose and wide in fact. At the superseded
/// `L = 5` configuration the harness reported `C_d = 23.7577`, and the smoothing ladder alone moved
/// the reported value across `7.70 … 47.27` without ever leaving this bracket.
///
/// The magnitude is not an isolated-cylinder `C_d`. It is inflated by the ~30 % blockage of the
/// periodic box and by the penalization integral counting the momentum sink over the whole smoothed
/// mask skirt rather than pressure plus friction on a sharp surface. The README's "Honest reading of
/// the absolute C_d" lists those causes plus the transient horizon and the Reynolds mismatch against
/// the DEC cross-reference. An earlier doc comment called this an "O(1) drag coefficient" bound; that
/// was wrong on both the value and the intent.
///
/// Tightening it to bracket a measured value (for example `[20, 30]`) would make it discriminating,
/// and that is a gate-bound change, deliberately not made here.
const DRAG_SANITY_MAX: f64 = 100.0;
/// Relative bound for judging a parameter ladder as settled.
///
/// Deliberately loose: the question these ladders answer is whether the drag settles *at all*, not
/// whether it settles tightly. A ladder outside this bound is reported through
/// [`LadderOutcome::NotConverging`] rather than silently passing.
const LADDER_TOL_REL: f64 = 0.05;

/// One bond cap's measured results.
pub struct BondRow {
    pub cap: usize,
    pub drag: f64,
    pub interior_max_speed: f64,
    pub divergence: f64,
}

/// One rung of a parameter ladder (η or mask smoothing width).
pub struct LadderRow {
    /// The swept parameter value.
    pub value: f64,
    pub drag: f64,
    pub interior_max_speed: f64,
}

/// Judge a ladder's drag sequence, normalising the tolerance by the drag scale so the bound is
/// relative rather than absolute.
fn judge(rows: &[LadderRow]) -> LadderOutcome {
    let drags: Vec<f64> = rows.iter().map(|r| r.drag).collect();
    let scale = drags
        .iter()
        .fold(0.0_f64, |a, d| a.max(d.abs()))
        .max(1.0e-12);
    LadderOutcome::judge(&drags, LADDER_TOL_REL * scale)
}

/// The maximum speed inside the body (mask > 0.9) at a run's final state — the no-slip diagnostic.
pub fn interior_max_speed(report: &Report<FloatType>, l: usize) -> f64 {
    let n = 1usize << l;
    // An accurate mask for the interior classification (geometry is independent of the solver bond cap).
    let accurate = config::trunc_bond(4096);
    let mask = config::cyl_mask(l, &accurate).expect("cylinder mask");
    let md = dequantize_2d(&mask, l, l).expect("dequantize mask");
    let us = report.final_field().expect("final u");
    let vs = report.series("final_v").expect("final v");
    let ms = md.as_slice();
    let mut m = 0.0f64;
    for k in 0..n * n {
        if Into::<f64>::into(ms[k]) > 0.9 {
            let (u, v) = (Into::<f64>::into(us[k]), Into::<f64>::into(vs[k]));
            m = m.max((u * u + v * v).sqrt());
        }
    }
    m
}

/// Render the accuracy-vs-bond table (stdout) and the cross-reference summary (stderr).
pub fn render(rows: &[BondRow]) {
    println!("Accuracy vs bond: immersed cylinder, drag from the penalization contraction");
    for (k, r) in rows.iter().enumerate() {
        let delta = if k == 0 {
            "   -- ".to_string()
        } else {
            format!("{:.2e}", (r.drag - rows[k - 1].drag).abs())
        };
        println!(
            "  bond <= {:>3}   C_d = {:.4}   |dC_d| = {}   interior_max|u| = {:.2e}   divergence = {:.2e}",
            r.cap, r.drag, delta, r.interior_max_speed, r.divergence,
        );
    }
    let finest = rows.last().expect("at least one bond cap");
    eprintln!(
        "\nfinest bond {}: C_d = {:.4}, interior max|u| = {:.2e} (free-stream {:.1})",
        finest.cap,
        finest.drag,
        finest.interior_max_speed,
        config::U_INF,
    );
    eprintln!(
        "DEC isolated-cylinder cross-reference (Re 100): C_d ~ {:.3} — disclaimed for periodic blockage",
        config::DEC_CD_REF,
    );
}

/// Render the two parameter ladders and their verdicts.
pub fn render_ladders(eta_rows: &[LadderRow], smooth_rows: &[LadderRow], cap: usize) {
    println!("\nParameter ladders at bond cap {cap} (the axes that move the reported drag)");

    println!("  eta ladder — the Brinkman eta -> 0 limit (Angot, Bruneau & Fabrie 1999):");
    for r in eta_rows {
        println!(
            "    eta = {:<7.4}  C_d = {:>8.4}   interior_max|u| = {:.2e}",
            r.value, r.drag, r.interior_max_speed
        );
    }
    println!("    verdict: {}", judge(eta_rows));

    println!("  mask smoothing ladder — a purely numerical width:");
    for r in smooth_rows {
        println!(
            "    delta = {:<5.1} cells  C_d = {:>8.4}   interior_max|u| = {:.2e}",
            r.value, r.drag, r.interior_max_speed
        );
    }
    println!("    verdict: {}", judge(smooth_rows));
}

/// Self-verification (exit nonzero on break): no-slip interior, bond saturation, physical drag, and
/// the two parameter ladders that constrain the reported drag.
///
/// BREAKING CONDITIONS: raise the no-slip floor above the penalization floor and gate 1 fails;
/// starve the bond cap and gate 2 fails (the bound is now ~6 orders above the measurement rather
/// than eleven); return a negative or NaN drag and gate 3 fails; make either swept parameter move
/// the drag without settling and its ladder reports `NOT CONVERGING`.
pub fn verify(rows: &[BondRow], eta_rows: &[LadderRow], smooth_rows: &[LadderRow]) -> bool {
    let mut ok = true;
    let finest = rows.last().expect("at least one bond cap");
    println!("\n--- immersed-cylinder gates ---");
    let mut gate = |label: &str, evidence: EvidenceClass, pass: bool, detail: String| {
        println!(
            "  [{}] [{evidence}] {label}: {detail}",
            if pass { "PASS" } else { "FAIL" }
        );
        if !pass {
            ok = false;
        }
    };

    // 1. No-slip: the interior velocity is at the penalization floor. Tripwire, and note it is
    //    invariant across the whole smoothing ladder, so it says nothing about the reported drag.
    gate(
        "no-slip enforced in the body interior",
        EvidenceClass::Tripwire,
        finest.interior_max_speed <= NO_SLIP_FLOOR * config::U_INF,
        format!(
            "interior max|u| {:.3e} vs floor {:.3e}",
            finest.interior_max_speed,
            NO_SLIP_FLOOR * config::U_INF
        ),
    );

    // 2. Bond saturation: the compression has converged. This says nothing about whether the
    //    compressed quantity is correct — that is what the parameter ladders below test.
    if rows.len() >= 2 {
        let prev = &rows[rows.len() - 2];
        let rel = (finest.drag - prev.drag).abs() / finest.drag.abs().max(1e-12);
        gate(
            "tensor-train compression saturated in bond",
            EvidenceClass::Tripwire,
            rel <= CONVERGENCE_BOUND,
            format!(
                "relative change {rel:.3e} between bond {} and {} vs bound {CONVERGENCE_BOUND:.1e}",
                prev.cap, finest.cap
            ),
        );
    }

    // 3. Blow-up guard: positive and finite. Not an O(1) claim — see DRAG_SANITY_MAX.
    gate(
        "drag positive and finite",
        EvidenceClass::Tripwire,
        finest.drag > 0.0 && finest.drag < DRAG_SANITY_MAX,
        format!("C_d {:.4} in (0, {DRAG_SANITY_MAX})", finest.drag),
    );

    // 4-5. The parameter ladders. A ladder that does not settle is reported as the result rather
    // than passed over: without an eta -> 0 limit the penalization integral has not been shown to
    // converge to a drag, and a C_d that tracks the mask smoothing width is reporting a numerical
    // choice. Both are real findings about this configuration, which is why they gate.
    for (name, rows_l) in [("eta", eta_rows), ("mask smoothing", smooth_rows)] {
        let outcome = judge(rows_l);
        gate(
            &format!("{name} ladder establishes a limit for C_d"),
            EvidenceClass::Reference,
            outcome.is_converged(),
            format!("{outcome}"),
        );
    }

    ok
}

/// The closing verdict on a successful verification.
pub fn summary() {
    println!(
        "\nImmersed cylinder verified: no-slip enforced, drag converges with bond dimension, and the"
    );
    println!(
        "streamwise drag is positive and finite (its magnitude reflects the smoothing skirt + blockage,"
    );
    println!("so the convergence trend is the result — cross-referenced to the DEC solver).");
}
