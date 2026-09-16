/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the geometric-QEC validation.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::constants::{LATTICE_SIDE, LDPC_BOUND};
use crate::{FloatType, NumberType};
use deep_causality_num::NaturalNumber;
use deep_causality_num::lower;
use deep_causality_quantum::{CheckMatrix, CheckVerdict, CssCode, LdpcItem, LdpcWeights, Screened};

pub fn print_header() {
    println!("=== QCL code path: the [[32, 2]] toric code, verified exactly ===\n");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Subject:   a {LATTICE_SIDE}x{LATTICE_SIDE} square torus, read as a CSS code");
    println!("           qubits on the edges, Z checks on the faces, X checks on the vertices\n");
}

/// The complex the code is read off.
pub fn print_complex(vertices: usize, edges: usize, faces: usize) {
    println!("[subject] {vertices} vertices, {edges} edges, {faces} faces");
    println!(
        "    no probes, no baseline, no evidence: the code subject offers validate stages only"
    );
    println!();
}

/// The four validation stages and what each examined.
pub fn print_stages<N: NaturalNumber, S>(screened: &Screened<FloatType, N, S>) {
    println!("[validate]");

    for (name, report) in screened.stages() {
        println!(
            "    {name:<24} {:?}  examined {}",
            report.verdict(),
            report.examined()
        );
    }
    println!();
}

/// What `derive_code` read off the complex.
pub fn print_derived_code(code: &CssCode<NumberType>) {
    println!("[derive_code]  [[n = {}, k = {}]]", code.n(), code.k());
    println!(
        "    {} Z checks of weight {}, {} X checks of weight {}, no distance claimed",
        code.z_generators().len(),
        code.z_generators()[0].weight(),
        code.x_generators().len(),
        code.x_generators()[0].weight()
    );
    println!();
    println!("    k is beta_1, the number of independent loops on the torus. The code's logical");
    println!("    qubit count is a topological invariant of the surface it was built on.");
    println!();
}

/// The weights against a bound the code meets.
pub fn print_weights(bound: usize, weights: &LdpcWeights<FloatType>) {
    println!(
        "[check_ldpc_weights]  bound {bound}: max column weight {}, max row weight {}, {} items examined, {:?}",
        weights.max_column_weight,
        weights.max_row_weight,
        weights.report.examined(),
        weights.report.verdict()
    );
}

/// The same check against a bound the code fails.
pub fn print_rejection(
    bound: usize,
    offender: (CheckMatrix, LdpcItem),
    margin: FloatType,
    weights: &LdpcWeights<FloatType>,
) {
    println!(
        "    bound {bound}: rejected at {offender:?} with margin {:.3} after {} items",
        lower(margin),
        weights.report.examined()
    );
    println!();
    println!("    The second bound is there so the first one means something. A suite that only");
    println!("    ever accepts says nothing about whether it would notice, and the rejection");
    println!("    names the generator rather than reporting that something somewhere failed.");
    println!();
}

/// The two structural checks.
pub fn print_structural_checks<N: NaturalNumber, S>(
    screened: &Screened<FloatType, N, S>,
    boundaries: usize,
) {
    let stages = screened.stages();

    println!(
        "[check_class_invariance]  {} (class, gate) pairs over {boundaries} boundaries each: Z-bar, S-bar, T-bar act on the class",
        stages[2].1.examined()
    );
    println!(
        "[check_clifford_action]   {} logical Hadamards swap Z-bar(g) <-> X-bar(g~), up to phase and stabilizers",
        stages[3].1.examined()
    );
    println!();
}

/// What the screen came to.
pub fn print_outcome(verdict: CheckVerdict, accepted: bool) {
    println!("Outcome");
    println!("    screen verdict            {verdict:?}");
    println!(
        "    every check accepted      {}",
        if accepted { "yes" } else { "NO" }
    );
    println!("    LDPC bound met            {LDPC_BOUND}");
    println!();
    println!("    Verified by exact F2 predicates, not simulated. The in-process simulator caps");
    println!("    at 24 qubits and this code has 32, so a state-vector check could not reach it.");
    println!("    The predicates do not need one: they decide over supports, and a support is a");
    println!("    finite object a computer can settle rather than approximate.");
}
