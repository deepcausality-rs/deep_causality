/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the Chern-number analysis.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{N_QUADRATURE, N_WILSON, nearest_chern_number};
use crate::{largest_departure_from_integer, largest_disagreement};
use deep_causality_num::lower;

/// One material phase: its mass parameter, the Chern number by each route, and what the phase
/// diagram calls it.
#[derive(Default, Clone, Debug)]
pub struct Phase {
    pub u: FloatType,
    pub quadrature: FloatType,
    pub wilson: FloatType,
    pub label: &'static str,
}

pub fn print_header() {
    println!("=== A topological insulator: the Chern number, two independent ways ===\n");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Model:     Qi-Wu-Zhang, H(k) = d(k).sigma");
    println!("           d(k) = (sin kx, sin ky, u + cos kx + cos ky)");
    println!(
        "Resolution: {N_QUADRATURE} Simpson panels per axis, {N_WILSON}x{N_WILSON} Wilson grid\n"
    );
}

/// The table, and what it establishes.
pub fn print_report(rows: &[Phase]) {
    println!("     u    | C (quadrature) | C (Wilson loop) |  C  | phase");
    println!("  --------+----------------+-----------------+-----+------------------------");

    for row in rows {
        println!(
            "  {:>6.1}  |   {:>11.6}  |   {:>12.6}  | {:>+2}  | {}",
            lower(row.u),
            lower(row.quadrature),
            lower(row.wilson),
            nearest_chern_number(row.quadrature),
            row.label
        );
    }

    println!();
    println!("Agreement");
    println!(
        "  largest gap between the two routes   {:>12.3e}",
        lower(largest_disagreement(rows))
    );
    println!(
        "  largest departure from an integer    {:>12.3e}",
        lower(largest_departure_from_integer(rows))
    );
    println!();
    println!("  The second number is the one that matters. Nothing in either calculation rounds");
    println!("  or snaps to an integer; both integrate a smooth function over a closed surface,");
    println!("  and an integer is what comes out. That is the quantisation, and it is why the");
    println!("  Hall conductance of such a material survives disorder that changes everything");
    println!("  else about it.");
    println!();
    println!("  The first number is what makes the second believable. The quadrature route");
    println!("  differentiates the d-vector and never forms a spinor; the Wilson route forms");
    println!("  spinors and never differentiates. They share the d-vector and nothing else.");
    println!();
    println!("  Phase diagram:  |u| > 2 -> C = 0;   0 < u < 2 -> C = +1;   -2 < u < 0 -> C = -1");
}
