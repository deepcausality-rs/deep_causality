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
use deep_causality_cfd::{Report, dequantize_2d};

/// Pinned no-slip floor: interior speed must fall below this fraction of the free-stream.
const NO_SLIP_FLOOR: f64 = 0.15;
/// Pinned drag-convergence bound: the relative change between the two finest bond caps.
const CONVERGENCE_BOUND: f64 = 0.10;
/// Pinned upper bound on a physical O(1) drag coefficient (sanity).
const DRAG_SANITY_MAX: f64 = 100.0;

/// One bond cap's measured results.
pub struct BondRow {
    pub cap: usize,
    pub drag: f64,
    pub interior_max_speed: f64,
    pub divergence: f64,
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

/// Self-verification (exit nonzero on break): no-slip interior, drag convergence with bond, physical drag.
pub fn verify(rows: &[BondRow]) -> bool {
    let mut ok = true;
    let finest = rows.last().expect("at least one bond cap");

    // 1. No-slip: the interior velocity is at the penalization floor.
    if finest.interior_max_speed > NO_SLIP_FLOOR * config::U_INF {
        eprintln!(
            "FAIL: no-slip not enforced — interior max|u| {:.3e} exceeds {:.3e}",
            finest.interior_max_speed,
            NO_SLIP_FLOOR * config::U_INF,
        );
        ok = false;
    }

    // 2. Drag converges as the bond cap rises: the finest-pair relative change is small.
    if rows.len() >= 2 {
        let prev = &rows[rows.len() - 2];
        let rel = (finest.drag - prev.drag).abs() / finest.drag.abs().max(1e-12);
        if rel > CONVERGENCE_BOUND {
            eprintln!(
                "FAIL: drag not converged — relative change {rel:.3} between bond {} and {} exceeds {CONVERGENCE_BOUND}",
                prev.cap, finest.cap,
            );
            ok = false;
        }
    }

    // 3. Physical drag: positive and finite (the absolute magnitude is inflated by the smoothing skirt
    //    + blockage — the convergence trend is the result, not the number).
    if !(finest.drag > 0.0 && finest.drag < DRAG_SANITY_MAX) {
        eprintln!("FAIL: drag {:.4} is not positive and finite", finest.drag);
        ok = false;
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
