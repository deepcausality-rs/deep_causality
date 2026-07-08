/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Display utilities for the GM recovery example.

use core::fmt::{Display, LowerExp};

use deep_causality_algebra::RealField;

use crate::pipeline::GmReport;

/// Pretty-prints a [`GmReport`] to stdout.
///
/// Lays the recovered values side-by-side with the published references
/// for both the geocentric gravitational parameter $GM_\oplus$ and the
/// derived planetary mass $M_\oplus = GM_\oplus / G$. The mass row makes
/// the headline result legible at a glance: *Earth's mass recovered from
/// clock time-dilation alone.*
///
/// Generic over the precision type so the same routine works for `f64`
/// and `Float106`.
pub fn print_gm_report<R>(report: &GmReport<R>)
where
    R: RealField + LowerExp + Display + From<f64> + Into<f64>,
{
    // Cast to f64 just for the human-readable percentage column. Native
    // Float106 Display would render both double-double components as a
    // composite — accurate, but not what a reader wants on a 4-decimal
    // percentage.
    let gm_err_pct: f64 = report.gm_relative_error.into() * 100.0;
    let mass_err_pct: f64 = report.mass_relative_error.into() * 100.0;

    println!("--- Recovery Report ---");
    println!("  Pair inversions:       {}", report.n_pairs);
    println!("  After MAD filter:      {}", report.n_after_mad);
    println!();
    println!("                       Recovered          Reference (JGM-3 / IERS 2010)");
    println!(
        "  GM_Earth  [m³/s²]    {:.6e}        {:.6e}",
        report.mean_gm, report.reference_gm
    );
    println!(
        "  M_Earth   [kg]       {:.6e}        {:.6e}",
        report.recovered_mass_kg, report.reference_mass_kg
    );
    println!();
    println!(
        "  Median GM:             {:.6e} m³/s²    σ = {:.3e} m³/s²",
        report.median_gm, report.std_gm
    );
    println!(
        "  Relative error (GM):   {:.4} %   ({:.3e})",
        gm_err_pct, report.gm_relative_error
    );
    println!(
        "  Relative error (M):    {:.4} %   ({:.3e})",
        mass_err_pct, report.mass_relative_error
    );
    println!();
    println!(
        "  Earth's mass recovered from satellite clock time-dilation alone, with an error of {:.4} %.",
        mass_err_pct
    );
    println!();
    println!("References:");
    println!("  Bjerhammar, A. (1975). Discrete approaches to the boundary value problem.");
    println!("  Vermeer, M. (1983). Chronometric levelling.");
    println!("  IERS Conventions (2010). Earth mass derived as M = GM/G.");
}
