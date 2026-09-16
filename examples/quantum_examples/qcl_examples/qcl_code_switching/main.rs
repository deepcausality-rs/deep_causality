/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! QCL-2: switching two logical qubits from `[[4,2,2]]` into the `[[8,2,2]]` toric code.
//!
//! No single code performs every gate transversally, so a machine that wants a universal set has to
//! move its logical qubits between codes. The move is a **gadget**: decode out of the first code and
//! encode into the second. It is where the logical information is briefly unprotected, and so it is
//! where a switch costs something.
//!
//! The low-level model encodes into code A, decodes, optionally depolarises one logical wire, then
//! encodes into code B and runs `Z̄` there. The middle model is code B's own model and the high
//! level is `Z̄` on two qubits. The first link is the gadget, aligned by the physical identity, so
//! its residual is exactly what the gadget cost; the second link is code B's abstraction.
//!
//! A noiseless gadget composes exactly. A depolarised one sits under the bound the law records, and
//! the run prints both so the cost is visible rather than assumed small.
//!
//! This is an example with checks, not a theorem: it claims the residuals it measures and the
//! bound the law records.

mod constants;
mod utils_print;

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_quantum::utils_tests::four_two_two;
use deep_causality_quantum::{
    CompositionLaw, LogicalGate, NumericCaps, QuantumError, code_switching, depolarizing_kraus,
};
use deep_causality_topology::LatticeComplex;

use crate::constants::{TORUS_SIDE, exactness_threshold, gadget_noise};
use utils_print::{print_clean, print_header, print_noisy, print_outcome, print_row};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; both codes, the
/// gadget and every residual recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

/// The count word the logical bases are computed over.
pub type NumberType = u64;

/// The composite law of the switch `A → B` for `Z̄(0)`, with or without gadget noise.
fn law_for<S>(noisy: bool) -> Result<CompositionLaw<S>, QuantumError>
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let code_a = four_two_two();
    let code_b = LatticeComplex::<2, S>::square_torus(TORUS_SIDE);

    let gadget = if noisy {
        depolarizing_kraus::<S>(gadget_noise::<S>())?
    } else {
        Vec::new()
    };

    Ok(
        code_switching::<NumberType, _, _, S>(&code_a, &code_b, &LogicalGate::Z(0), gadget)?
            .compose(&NumericCaps::default())?
            .law,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // The noiseless switch: the gadget is the identity on the logical space, so it costs nothing.
    let clean = law_for::<FloatType>(false)?;
    let clean_is_exact = clean.measured() < exactness_threshold::<FloatType>();
    print_clean(&clean, clean_is_exact);

    // The same switch with one logical wire depolarised. The noise lands entirely in the first
    // link, which is what the alignment by the physical identity is for.
    let noisy = law_for::<FloatType>(true)?;
    let second_link_exact = noisy.rows[0].epsilon_second < exactness_threshold::<FloatType>();
    print_noisy(&noisy, second_link_exact);

    println!("[noisy gadget] at the shipped precisions");
    let f32_law = law_for::<f32>(true)?;
    let f64_law = law_for::<f64>(true)?;
    let wide_law = law_for::<Float106>(true)?;

    print_row("f32", &f32_law);
    print_row("f64", &f64_law);
    print_row("Float106", &wide_law);

    let every_law_holds =
        clean.holds() && noisy.holds() && f32_law.holds() && f64_law.holds() && wide_law.holds();

    print_outcome(every_law_holds, clean_is_exact);

    if !(every_law_holds && clean_is_exact && second_link_exact) {
        return Err("a composition law or an exactness check failed".into());
    }

    Ok(())
}
