/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the concatenated-code example.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lower};
use deep_causality_quantum::CompositionLaw;

pub fn print_header() {
    println!("=== QCL-2 chain: the [[4,2,2]] code concatenated with itself ===\n");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Chain      L: 8 physical qubits, two blocks of the code");
    println!("           M: the inner code's 4 qubits");
    println!("           H: 2 logical qubits");
    println!("Links      first: the outer recovery per block.  second: the inner abstraction\n");
}

/// One gate's composite law at the working precision.
pub fn print_gate(name: &str, law: &CompositionLaw<FloatType>) {
    println!("[{name}] at FloatType");
    print!("{law}");
}

/// Whether the composite came out exact, and what that rests on.
pub fn print_exactness(exact: bool) {
    if exact {
        println!("    exact: two residual-zero links compose to residual zero (Proposition 17)");
    } else {
        println!("    NOT exact: a residual survived that Proposition 17 says should not");
    }
    println!();
}

/// One precision's row.
pub fn print_row<S>(name: &str, law: &CompositionLaw<S>)
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let row = &law.rows[0];

    println!(
        "    {name:>8}: e1 = {:.2e}, e2 = {:.2e}, |t1|_pre = {:.6}, |t2|_post = {:.6}, bound = {:.2e}, measured = {:.2e}, {}",
        lower(row.epsilon_first),
        lower(row.epsilon_second),
        lower(row.pre),
        lower(row.post),
        lower(row.bound),
        lower(row.measured),
        if law.holds() { "holds" } else { "VIOLATED" }
    );
}

/// The gate that crosses code blocks, and what the construction did with it.
pub fn print_refusal(reason: Option<&str>) {
    println!("[CZ-bar(0, 1)] the inner CZ-bar pairs a qubit of each outer block");

    match reason {
        Some(reason) => println!("    refused: {reason}"),
        None => println!("    UNEXPECTED: a gate across blocks has no transversal gadget here"),
    }
    println!();
}

/// What the run came to.
pub fn print_outcome(every_law_holds: bool, every_square_exact: bool, refused: bool) {
    println!("Outcome");
    println!(
        "  every composition law held      {}",
        yes_no(every_law_holds)
    );
    println!(
        "  every transversal Pauli exact   {}",
        yes_no(every_square_exact)
    );
    println!("  the cross-block gate refused    {}", yes_no(refused));
    println!();
    println!("  The third line matters as much as the first two. A construction that returned");
    println!("  something for a gate it has no gadget for would compose cleanly and be wrong,");
    println!("  and nothing in the law would say so.");
    println!();
    println!("  This is an example with checks, not a theorem.");
}

fn yes_no(ok: bool) -> &'static str {
    if ok { "yes" } else { "NO" }
}
