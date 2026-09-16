/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the repetition-code run.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{N_QUBITS, Syndrome, ZERO, norm_squared, parity};
use deep_causality_multivector::HilbertState;
use deep_causality_num::lower;

/// Basis labels for a three-qubit register, least significant bit on the left so that qubit 0 is
/// written first.
fn ket(index: usize) -> String {
    (0..N_QUBITS)
        .map(|q| if (index >> q) & 1 == 1 { '1' } else { '0' })
        .collect()
}

/// The basis states a register actually occupies, with their amplitudes.
fn support(state: &HilbertState<FloatType>) -> String {
    state
        .as_inner()
        .data()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.re != ZERO || c.im != ZERO)
        .map(|(i, c)| format!("{:.3}|{}>", lower(c.re), ket(i)))
        .collect::<Vec<_>>()
        .join(" + ")
}

pub fn print_header() {
    println!("=== A quantum counterfactual: what the decoder was reading ===\n");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Code:      the three-qubit repetition code, one logical qubit in three\n");
}

/// The encoding.
pub fn print_encoding(protected: &HilbertState<FloatType>) {
    println!("Encoding");
    println!("  logical      0.600|0> + 0.800|1>");
    println!("  encoded      {}", support(protected));
    println!("  norm^2       {:.9}", lower(norm_squared(protected)));
    println!();
    println!("  A lone qubit cannot be checked: any measurement that would reveal a flip also");
    println!("  reveals the amplitudes and collapses the superposition. Three qubits leave");
    println!("  questions about how they relate to each other, which is what a syndrome asks.\n");
}

/// The error, as a permutation of the same amplitudes.
pub fn print_error(
    protected: &HilbertState<FloatType>,
    corrupted: &HilbertState<FloatType>,
    qubit: usize,
) {
    println!("The error: an X gate on qubit {qubit}");
    println!("  before       {}", support(protected));
    println!("  after        {}", support(corrupted));
    println!("  norm^2       {:.9}", lower(norm_squared(corrupted)));
    println!();
    println!("  The same eight numbers came out in a different order. Nothing was substituted,");
    println!("  and the decoder is never told which qubit this was.\n");
}

/// The parity checks, before and after, showing they do not depend on the amplitudes.
pub fn print_parities(protected: &HilbertState<FloatType>, corrupted: &HilbertState<FloatType>) {
    println!("The syndrome: two parity measurements");
    println!("  check        clean      corrupted");
    println!(
        "  <Z0 Z1>    {:>+7.3}      {:>+7.3}",
        lower(parity(protected, 0, 1)),
        lower(parity(corrupted, 0, 1))
    );
    println!(
        "  <Z1 Z2>    {:>+7.3}      {:>+7.3}",
        lower(parity(protected, 1, 2)),
        lower(parity(corrupted, 1, 2))
    );
    println!();
    println!("  Every value is exactly +1 or -1, and none of them is 0.6 or 0.8. That is the");
    println!("  property the code is built on: the measurement learns that two qubits disagree");
    println!("  without learning what either of them holds.\n");
}

/// One recovery, named by the syndrome that drove it.
pub fn print_recovery(
    label: &str,
    syndrome: Syndrome,
    recovered: &HilbertState<FloatType>,
    fidelity: FloatType,
) {
    let names = match crate::model::decode(syndrome) {
        Some(q) => format!("qubit {q}"),
        None => "no error".to_string(),
    };

    println!("Recovery from {label}");
    println!(
        "  syndrome     ({}, {})  ->  {}",
        pair(syndrome.first_pair_differs),
        pair(syndrome.second_pair_differs),
        names
    );
    println!("  state        {}", support(recovered));
    println!("  fidelity     {:.9}", lower(fidelity));
    println!();
}

/// What the two runs came to.
pub fn print_outcome(observed: FloatType, intervened: FloatType, uncorrected: FloatType) {
    println!("Outcome");
    println!("  fidelity to the protected state");
    println!("    observed syndrome        {:.9}", lower(observed));
    println!("    intervened syndrome      {:.9}", lower(intervened));
    println!("    no correction at all     {:.9}", lower(uncorrected));
    println!();
    println!("  The two pipelines ran the same four steps on the same corrupted register and");
    println!("  differ by one line. Reading the syndrome the measurement produced restores the");
    println!("  state exactly. Reading a syndrome an intervention forced applies a correction to");
    println!("  the wrong qubit, and leaves a state orthogonal to the one being protected: no");
    println!("  worse in norm, and no use at all.");
    println!();
    println!("  That is the counterfactual. The recovery is caused by the syndrome, and the run");
    println!("  shows it by substituting the syndrome and watching the recovery follow.");
}

fn pair(differs: bool) -> &'static str {
    if differs { "differ" } else { "agree " }
}
