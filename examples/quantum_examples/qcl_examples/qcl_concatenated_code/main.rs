/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! QCL-2: the `[[4,2,2]]` code concatenated with itself, as two abstractions composed.
//!
//! The low-level model is eight physical qubits carrying two blocks of the code; the middle model is
//! the inner code's four qubits; the high level is two logical qubits. The first link is the outer
//! recovery applied per block and the second is the inner code's abstraction, so the composite is
//! the concatenated code and `Z̄` and `X̄` compose through it exactly (Proposition 17).
//!
//! A gate across code blocks has no transversal gadget in this construction, and the run shows the
//! refusal by name rather than asserting that one exists.
//!
//! This is an example with checks, not a theorem: it claims the residuals it measures and the
//! bound the law records.

mod constants;
mod utils_print;

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_quantum::utils_tests::four_two_two;
use deep_causality_quantum::{
    CompositionLaw, LogicalGate, NumericCaps, QuantumError, concatenated_code,
};

use crate::constants::exactness_threshold;
use utils_print::{
    print_exactness, print_gate, print_header, print_outcome, print_refusal, print_row,
};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the code, the
/// gates and every residual recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

/// The count word the logical basis is computed over.
pub type NumberType = u64;

/// The gates the run composes through the concatenation.
const TRANSVERSAL_GATES: [LogicalGate; 2] = [LogicalGate::Z(0), LogicalGate::X(1)];

/// The composite law for one logical gate at one precision.
fn law_for<S>(gate: &LogicalGate) -> Result<CompositionLaw<S>, QuantumError>
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let complex = four_two_two();

    Ok(
        concatenated_code::<NumberType, _, _, S>(&complex, &complex, gate)?
            .compose(&NumericCaps::default())?
            .law,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let mut every_law_holds = true;
    let mut every_square_exact = true;

    for gate in TRANSVERSAL_GATES {
        let law = law_for::<FloatType>(&gate)?;
        let row = &law.rows[0];

        every_law_holds &= law.holds();

        // Both links are exact for a transversal Pauli, so the composite is. The threshold is in
        // machine epsilons at the precision in force, which is what makes the same claim testable
        // at every scalar.
        let exact = row.measured < exactness_threshold::<FloatType>()
            && row.bound < exactness_threshold::<FloatType>();
        every_square_exact &= exact;

        print_gate(&gate.name(), &law);
        print_exactness(exact);

        println!("[{}] at the shipped precisions", gate.name());
        let f32_law = law_for::<f32>(&gate)?;
        let f64_law = law_for::<f64>(&gate)?;
        let wide_law = law_for::<Float106>(&gate)?;

        every_law_holds &= f32_law.holds() && f64_law.holds() && wide_law.holds();

        print_row("f32", &f32_law);
        print_row("f64", &f64_law);
        print_row("Float106", &wide_law);
        println!();
    }

    // A gate pairing a qubit of each outer block. There is no transversal gadget for it here, and
    // the construction says so rather than returning something that does not hold.
    let complex = four_two_two();
    let across_blocks = concatenated_code::<NumberType, _, _, FloatType>(
        &complex,
        &complex,
        &LogicalGate::Cz(0, 1),
    );

    let refused = match across_blocks {
        Ok(_) => {
            print_refusal(None);
            false
        }
        Err(e) => {
            print_refusal(Some(&format!("{e}")));
            true
        }
    };

    print_outcome(every_law_holds, every_square_exact, refused);

    // The three are the example's claims about itself. A run that prints one of them as NO has
    // shown the reader what failed; returning the failure is what lets a script see it too.
    if !(every_law_holds && every_square_exact && refused) {
        return Err(
            "a composition law, an exactness check or the cross-block refusal failed".into(),
        );
    }

    Ok(())
}
