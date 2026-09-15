/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A concatenated code as a chain of two abstractions.
//!
//! The hand-built `[[4,2,2]]` code is concatenated with itself: the inner code's four physical
//! qubits are grouped into two blocks of two, and each block is encoded by the outer code, eight
//! physical qubits in all. The first link takes the eight-qubit model to the four-qubit one through
//! the outer recovery on each block; the second link is the inner code's own abstraction onto the
//! two logical qubits. `Abstraction::compose` pastes the two squares and records the law
//! `ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂` with both constants computed. For `Z̄` and `X̄` both
//! links are exact and so is the composite, which is Lorenz & Tull's Proposition 17 with the Lean
//! statement in `lean/DeepCausalityFormal/Quantum/Abstraction.lean`. `CZ̄` of the inner code pairs
//! a qubit of each block, and a gate across code blocks is refused by name: this construction
//! carries no transversal gadget between blocks.
//!
//! This is an example with checks, not a theorem: it claims the residuals it measures and the
//! bound the law records.

mod constants;

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive, lower};
use deep_causality_quantum::utils_tests::four_two_two;
use deep_causality_quantum::{CompositionLaw, LogicalGate, NumericCaps, concatenated_code};

use crate::constants::EXACT_AT_F64;

/// The working type. Switch it to `f32` or `Float106`; nothing below changes.
pub type FloatType = f64;

/// The count word the logical basis is computed over.
pub type NumberType = u64;

/// The composite law for one logical gate at one precision.
fn law_for<S>(gate: &LogicalGate) -> CompositionLaw<S>
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let complex = four_two_two();
    concatenated_code::<NumberType, _, _, S>(&complex, &complex, gate)
        .expect("the [[4,2,2]] code concatenates with itself for a single-qubit gate")
        .compose(&NumericCaps::default())
        .expect("both links have Io squares under the default caps")
        .law
}

/// One precision's report for one gate: the display boundary is the only place `f64` appears.
fn report<S>(name: &str, gate: &LogicalGate)
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let law = law_for::<S>(gate);
    let row = &law.rows[0];
    println!(
        "    {name:>8}: ε₁ = {:.2e}, ε₂ = {:.2e}, ‖τ₁‖_pre = {:.6}, ‖τ₂‖_post = {:.6}, bound = {:.2e}, measured = {:.2e}, {}",
        lower(row.epsilon_first),
        lower(row.epsilon_second),
        lower(row.pre),
        lower(row.post),
        lower(row.bound),
        lower(row.measured),
        if law.holds() { "holds" } else { "violated" }
    );
    assert!(law.holds(), "the composition law holds at every precision");
}

fn main() {
    println!("=== QCL-2 chain: the [[4,2,2]] code concatenated with itself ===\n");
    println!("[chain] L: 8 physical qubits, M: the inner code's 4 qubits, H: 2 logical qubits");
    println!(
        "        first link: outer recovery per block; second link: the inner code's abstraction\n"
    );

    for gate in [LogicalGate::Z(0), LogicalGate::X(1)] {
        let law = law_for::<FloatType>(&gate);
        println!("[{}] at FloatType", gate.name());
        print!("{law}");
        let row = &law.rows[0];
        assert!(
            lower(row.measured) < EXACT_AT_F64 && lower(row.bound) < EXACT_AT_F64,
            "both links are exact for a transversal Pauli, so the composite is"
        );
        println!("    exact: two residual-zero links compose to residual zero (Proposition 17)\n");
        println!("[{}] at the three shipped precisions", gate.name());
        report::<f32>("f32", &gate);
        report::<f64>("f64", &gate);
        report::<Float106>("Float106", &gate);
        println!();
    }

    println!("[CZ̄(0, 1)] the inner CZ̄ pairs a qubit of each outer block");
    let complex = four_two_two();
    match concatenated_code::<NumberType, _, _, FloatType>(
        &complex,
        &complex,
        &LogicalGate::Cz(0, 1),
    ) {
        Ok(_) => panic!("a gate across blocks has no transversal gadget here"),
        Err(e) => println!("    refused: {e}\n"),
    }
    println!("=== done: an example with checks, not a theorem ===");
}
