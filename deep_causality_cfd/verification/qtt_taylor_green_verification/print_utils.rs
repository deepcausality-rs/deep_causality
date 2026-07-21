/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Measurement + display + self-verification for the QTT Taylor–Green run. `main` runs the CfdFlow
//! march and hands the owned reports here; this layer measures them against the analytic reference,
//! renders the refinement ladder (CSV on stdout, summary on stderr), and gates the published-reference
//! checks (exit nonzero on break).
//!
//! Values cross to `f64` here, at the display/measurement boundary.

use crate::FloatType;
use crate::config;
use deep_causality_cfd::{EvidenceClass, Report};

/// Pinned finest-grid bound on the max-norm field error vs. the analytic decay.
const FINEST_ERR_BOUND: f64 = 2.0e-4;
/// Pinned minimum observed spatial-convergence order (centered FD + spectral projection → ~2).
const MIN_ORDER: f64 = 1.8;
/// Pinned bound on the convection-operator error vs. the closed form `−½ sin(2x)`.
const CONVECTION_BOUND: f64 = 1.0e-2;
/// Bounds on the ratio of the **computed** convection amplitude to the analytic one. The analytic
/// amplitude of `−½ sin(2x)` is exactly ½, so a correct operator lands near 1.0; a no-op term lands
/// at 0. Wide enough to absorb the discretisation error the `CONVECTION_BOUND` gate already sizes,
/// narrow enough that a missing or mis-scaled nonlinear term fails.
const AMP_RATIO_LO: f64 = 0.9;
const AMP_RATIO_HI: f64 = 1.1;
/// Pinned bound on the post-projection divergence residual.
const DIVERGENCE_BOUND: f64 = 1.0e-6;

/// One refinement level's measured results (at the display boundary, in `f64`).
pub struct LevelResult {
    pub n: usize,
    pub max_err: f64,
    pub l2: f64,
    pub bond: usize,
    pub dense: usize,
    pub divergence: f64,
}

/// Measure each `(L, report)` against the analytic decayed Taylor–Green vortex: the max- and L2-norm
/// field error (from the report's final `(u, v)` fields), the maximum bond dimension and the dense
/// element count it compresses, and the final divergence residual.
pub fn measure(runs: &[(usize, Report<FloatType>)]) -> Vec<LevelResult> {
    runs.iter().map(|(l, r)| measure_one(*l, r)).collect()
}

fn measure_one(l: usize, report: &Report<FloatType>) -> LevelResult {
    let n = 1usize << l;
    let dx = 2.0 * std::f64::consts::PI / n as f64;
    let d = Into::<f64>::into(config::decay());
    let us = report.final_field().expect("final u field");
    let vs = report.series("final_v").expect("final v series");

    let mut max_err = 0.0f64;
    let mut l2 = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            let (x, y) = (i as f64 * dx, j as f64 * dx);
            let eu = (Into::<f64>::into(us[i * n + j]) - config::tg_u(x, y) * d).abs();
            let ev = (Into::<f64>::into(vs[i * n + j]) - config::tg_v(x, y) * d).abs();
            max_err = max_err.max(eu).max(ev);
            l2 += eu * eu + ev * ev;
        }
    }
    l2 = (l2 / (2.0 * (n * n) as f64)).sqrt();

    let bond = Into::<f64>::into(
        *report
            .series("bond")
            .expect("bond series")
            .last()
            .expect("bond sample"),
    ) as usize;
    let divergence = Into::<f64>::into(
        *report
            .series("divergence")
            .expect("divergence series")
            .last()
            .expect("divergence sample"),
    );

    LevelResult {
        n,
        max_err,
        l2,
        bond,
        dense: n * n,
        divergence,
    }
}

/// Render the verification report as a labeled, human-readable summary on stdout (in the style of the
/// other verification examples): the convergence ladder, the convection-operator check, and the MPS
/// compression.
pub fn render(results: &[LevelResult], convection: (f64, f64, f64)) {
    let finest = results.last().expect("at least one level");

    println!("Convergence: refinement ladder vs the analytic e^(-2 nu t) decay");
    for (k, r) in results.iter().enumerate() {
        let order = if k == 0 {
            "  -- ".to_string()
        } else {
            format!("{:.2}", observed_order(&results[k - 1], r))
        };
        println!(
            "  N = {:>3}   max_err = {:.3e}   l2_err = {:.3e}   order = {}   bond = {:>3}   divergence = {:.2e}",
            r.n, r.max_err, r.l2, order, r.bond, r.divergence,
        );
    }
    if results.len() >= 2 {
        let order = observed_order(&results[results.len() - 2], finest);
        println!("  observed order = {order:.2} (centered FD + spectral projection -> 2)\n");
    } else {
        println!();
    }

    let (conv_err, conv_amp, conv_amp_ref) = convection;
    println!("Convection: nonlinear u.grad(u), u-component vs the closed form -1/2 sin(2x)");
    println!(
        "  max abs error = {conv_err:.3e}   (computed amplitude {conv_amp:.3} vs analytic {conv_amp_ref:.3})"
    );
    println!(
        "  (single-mode TG's convective term is a pure gradient the projection removes -- checked directly)\n"
    );

    println!("Compression: MPS bond vs dense storage at the finest grid");
    println!(
        "  bond {} vs {} dense elements = {:.1}x",
        finest.bond,
        finest.dense,
        finest.dense as f64 / finest.bond.max(1) as f64,
    );
}

/// The closing prose printed on a successful verification — the verdict, in the style of the other
/// verification examples' summaries.
pub fn summary() {
    println!(
        "\nTaylor-Green vortex reproduced: 2nd-order convergence to the analytic e^(-2 nu t) decay,"
    );
    println!("correct nonlinear convection, and divergence-free to machine precision.");
}

/// Self-verification against the **published reference** (Taylor & Green, 1937) plus the
/// convection-operator closed form. Gates:
/// 1. the field error converges (strictly decreases under refinement) to within [`FINEST_ERR_BOUND`];
/// 2. the finest-pair observed order is at least [`MIN_ORDER`] (2nd-order scheme);
/// 3. the nonlinear convection matches the closed form to [`CONVECTION_BOUND`] with non-zero amplitude
///    (the term single-mode TG masks);
/// 4. the post-projection divergence stays below [`DIVERGENCE_BOUND`] (incompressibility).
///
/// Returns `false` on any violation; `main` exits nonzero.
pub fn verify(results: &[LevelResult], convection: (f64, f64, f64)) -> bool {
    let mut ok = true;
    println!("\n--- Taylor-Green gates ---");
    let mut gate = |label: &str, evidence: EvidenceClass, pass: bool, detail: String| {
        println!(
            "  [{}] [{evidence}] {label}: {detail}",
            if pass { "PASS" } else { "FAIL" }
        );
        if !pass {
            ok = false;
        }
    };

    // 1. Monotone convergence + finest bound, against the closed-form e^(-2 nu t) decay.
    //    Reference class: the comparison target is the analytic solution, so clearing it is
    //    evidence about the discretisation's accuracy, not merely about drift.
    let monotone = results
        .windows(2)
        .all(|pair| pair[1].max_err < pair[0].max_err);
    gate(
        "error decreases under refinement",
        EvidenceClass::Reference,
        monotone,
        results
            .iter()
            .map(|r| format!("N={} {:.2e}", r.n, r.max_err))
            .collect::<Vec<_>>()
            .join(" -> "),
    );
    let finest = results.last().expect("at least one level");
    gate(
        "finest-grid error within bound",
        EvidenceClass::Reference,
        finest.max_err <= FINEST_ERR_BOUND,
        format!(
            "N={} error {:.3e} vs bound {:.3e} (vs the analytic decay)",
            finest.n, finest.max_err, FINEST_ERR_BOUND
        ),
    );

    // 2. Observed order of the finest pair, against the theoretical 2 for centered FD + spectral
    //    projection. The expectation is theoretical, not a pinned measurement.
    if results.len() >= 2 {
        let order = observed_order(&results[results.len() - 2], finest);
        gate(
            "observed spatial order",
            EvidenceClass::Reference,
            order >= MIN_ORDER,
            format!("{order:.3} >= {MIN_ORDER} (theoretical 2; 2nd order in space, 1st in time)"),
        );
    }

    // 3. Convection operator vs closed form.
    //
    // The amplitude check reads the COMPUTED field and compares it to the analytic amplitude. Taken
    // from the closed form alone (as it was) `conv_amp` is identically 0.5 on any grid, so
    // `conv_amp <= 0.0` admitted every possible solver output — including an all-zero convection
    // term — and could never fail.
    //
    // BREAKING CONDITION: zero the convection field and the ratio goes to 0, failing this gate.
    let (conv_err, conv_amp, conv_amp_ref) = convection;
    let amp_ratio = conv_amp / conv_amp_ref;
    gate(
        "convection amplitude matches the closed form",
        EvidenceClass::Reference,
        amp_ratio > AMP_RATIO_LO && amp_ratio < AMP_RATIO_HI,
        format!(
            "computed {conv_amp:.3e} is {amp_ratio:.3}x the analytic {conv_amp_ref:.3e}, \
             band [{AMP_RATIO_LO}, {AMP_RATIO_HI}]"
        ),
    );
    gate(
        "convection error within bound",
        EvidenceClass::Reference,
        conv_err <= CONVECTION_BOUND,
        format!("{conv_err:.3e} vs bound {CONVECTION_BOUND:.3e} (closed form -1/2 sin 2x)"),
    );

    // 4. Incompressibility. Tripwire: div(project(u)) = 0 is an identity of the spectral projector
    //    (projection.rs documents the eigenvalue chosen to make it exact), so this residual
    //    measures tensor-train truncation fidelity rather than a physical property.
    gate(
        "post-projection divergence at the truncation floor",
        EvidenceClass::Tripwire,
        finest.divergence <= DIVERGENCE_BOUND,
        format!(
            "{:.3e} vs bound {:.3e} (projector identity; this measures TT truncation)",
            finest.divergence, DIVERGENCE_BOUND
        ),
    );

    ok
}

/// Observed spatial-convergence order between two levels: `log2(err_coarse / err_fine)` (grid doubles
/// each level).
fn observed_order(coarse: &LevelResult, fine: &LevelResult) -> f64 {
    (coarse.max_err / fine.max_err).log2()
}
