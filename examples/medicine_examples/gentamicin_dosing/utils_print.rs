/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Printing for the gentamicin dosing example. Values cross to `f64` here, at the display boundary.

use crate::FloatType;
use crate::model::{
    Level, PEAK_MAX_MG_L, PEAK_MIN_MG_L, Report, TROUGH_GOAL_MG_L, TROUGH_MAX_MG_L,
};
use deep_causality_num::lower;

/// The dose starts at 08:00.
const DOSE_START_MIN: f64 = 8.0 * 60.0;

fn f(x: FloatType) -> f64 {
    lower(x)
}

/// Wall-clock time of `hour` hours after the 08:00 dose.
fn clock(hour: FloatType) -> String {
    let minutes = (DOSE_START_MIN + f(hour) * 60.0).round() as i64;
    format!("{:02}:{:02}", (minutes / 60) % 24, minutes % 60)
}

fn level(l: &Level) -> String {
    format!("{:.2} mg/L at {}", f(l.mg_l), clock(l.hour))
}

pub fn print_report(report: &Report) {
    let rec = &report.recommendation;
    let course = &rec.course;
    let measured = &course.peak.elimination.measured;
    let k = f(course.peak.elimination.k_per_h);
    println!("=== Gentamicin dosing from two serum levels ===\n");
    println!(
        "Regimen: one dose every {:.0} h, infused {} to {}",
        f(measured.interval_h),
        clock(FloatType::default()),
        clock(measured.infusion_end_h)
    );
    println!(
        "Measured: {} and {}\n",
        level(&measured.first),
        level(&measured.second)
    );
    println!(
        "1. Elimination rate:  k = {k:.4} /h, half-life {:.1} h",
        std::f64::consts::LN_2 / k
    );
    println!(
        "2. Peak (backward):   {}   target {:.0}-{:.0} mg/L: {}",
        level(&course.peak.peak),
        f(PEAK_MIN_MG_L),
        f(PEAK_MAX_MG_L),
        verdict(rec.peak_in_range)
    );
    println!(
        "3. Trough (forward):  {}   target at most {:.0} mg/L: {}",
        level(&course.trough),
        f(TROUGH_MAX_MG_L),
        verdict(rec.trough_in_range)
    );
    match rec.interval_h {
        Some(tau) => println!(
            "4. Interval:          the peak falls to {:.0} mg/L after {:.1} h; dose every {:.0} h",
            f(TROUGH_GOAL_MG_L),
            f(rec.interval_to_goal_h),
            f(tau)
        ),
        None => println!(
            "4. Interval:          the peak falls to {:.0} mg/L after {:.1} h, beyond every standard interval",
            f(TROUGH_GOAL_MG_L),
            f(rec.interval_to_goal_h)
        ),
    }
    println!(
        "\nCheck: the law run forward from the peak returns {} (difference {:.1e} mg/L)",
        level(&report.returned),
        (f(report.returned.mg_l) - f(measured.first.mg_l)).abs()
    );
}

fn verdict(in_range: bool) -> &'static str {
    if in_range { "in range" } else { "OUT OF RANGE" }
}
