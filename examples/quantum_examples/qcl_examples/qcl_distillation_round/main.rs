/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A distillation round as a chain of two abstractions, the case Lorenz & Tull defer (§7.1).
//!
//! The low-level model encodes two logical qubits into the hand-built `[[4,2,2]]` code, depolarises
//! every physical qubit with probability `p`, and runs the encoded magic-state rotation `T̄ H̄` on
//! both logical qubits. The middle model is the same without the noise, and the high level is
//! `T H` on two qubits. The first link aligns the noisy and the clean physical models by the
//! identity, so its residual is the noise; the second link is the code's abstraction. After the
//! composite's `τ`, the ideal recovery, the measured residual is the noise the recovery does not
//! remove, which is what a distillation round has to drive down, and the law bounds it by
//! `‖τ₂‖_post · ε₁`. The construction is a non-strict quantum-to-quantum abstraction.
//!
//! This is an example with checks, not a theorem: it claims the residuals it measures and the
//! bound the law records.

mod constants;
mod utils_print;

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_quantum::utils_tests::four_two_two;
use deep_causality_quantum::{CompositionLaw, NumericCaps, QuantumError, distillation_round};

use crate::constants::{COMPARISON_INDEX, NOISE_LABELS, noise_sweep};
use utils_print::{print_header, print_outcome, print_round, print_row};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the code, the
/// noise and every residual recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

/// The count word the logical basis is computed over.
pub type NumberType = u64;

/// The composition law for one depolarising probability.
fn law_for<S>(p: S) -> Result<CompositionLaw<S>, QuantumError>
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    Ok(distillation_round::<NumberType, _, S>(&four_two_two(), p)?
        .compose(&NumericCaps::default())?
        .law)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let sweep = noise_sweep::<FloatType>();
    let mut every_law_holds = true;

    for (p, label) in sweep.into_iter().zip(NOISE_LABELS) {
        let law = law_for::<FloatType>(p)?;
        every_law_holds &= law.holds();

        print_round(label, &law);
    }

    // The same round at each shipped precision. The probability is rebuilt from its fraction at
    // every one of them, so what the rows compare is the arithmetic and not a widened literal.
    let label = NOISE_LABELS[COMPARISON_INDEX];
    println!("[p = {label}] at the shipped precisions");

    let f32_law = law_for::<f32>(noise_sweep::<f32>()[COMPARISON_INDEX])?;
    let f64_law = law_for::<f64>(noise_sweep::<f64>()[COMPARISON_INDEX])?;
    let wide_law = law_for::<Float106>(noise_sweep::<Float106>()[COMPARISON_INDEX])?;

    every_law_holds &= f32_law.holds() && f64_law.holds() && wide_law.holds();

    print_row("f32", &f32_law);
    print_row("f64", &f64_law);
    print_row("Float106", &wide_law);

    print_outcome(every_law_holds);

    if !every_law_holds {
        return Err("a composition law was violated".into());
    }

    Ok(())
}
