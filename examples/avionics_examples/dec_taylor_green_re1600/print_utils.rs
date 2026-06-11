/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Display utilities for the Re-1600 Taylor–Green run.

use deep_causality_num::RealField;

use crate::model::Report;

/// CSV on stdout (machine-readable artifact), summary on stderr (human).
///
/// Generic over the precision type. The computation carries `R` end to
/// end; the cast to `f64` here is presentation only — native `Float106`
/// `Display` would render both double-double components as a composite,
/// which is accurate but not what a CSV consumer or plot script wants.
pub fn print_csv<R>(report: &Report<R>)
where
    R: RealField + Into<f64>,
{
    println!("t_star,kinetic_energy_per_vol,dissipation_rate");
    for s in &report.series {
        println!(
            "{:.4},{:.8},{:.8}",
            Into::<f64>::into(s.t_star),
            Into::<f64>::into(s.energy_per_vol),
            Into::<f64>::into(s.dissipation)
        );
    }

    if let (Some(first), Some(last)) = (report.series.first(), report.series.last()) {
        let peak = report.series.iter().fold((0.0_f64, 0.0_f64), |(t, d), s| {
            let sd: f64 = s.dissipation.into();
            if sd > d {
                (s.t_star.into(), sd)
            } else {
                (t, d)
            }
        });
        let e0: f64 = first.energy_per_vol.into();
        let e_t: f64 = last.energy_per_vol.into();
        eprintln!(
            "\nmarched to t* = {:.2}: E*/E0 = {:.4}, peak dissipation {:.6} at t* = {:.2}",
            Into::<f64>::into(last.t_star),
            e_t / e0,
            peak.1,
            peak.0
        );
        eprintln!(
            "compare the dissipation column against the published Re-1600 DNS curve (references.md)."
        );
    }
}
