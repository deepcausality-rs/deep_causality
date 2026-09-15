/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Code switching as a chain of two abstractions.
//!
//! The low-level model encodes two logical qubits into code A, the hand-built `[[4,2,2]]`, decodes
//! them again, and encodes them into code B, the `[[8,2,2]]` toric code, before running B's
//! program. The decode-and-re-encode is the switching gadget, and it is the low-level query of the
//! first link, which aligns the gadget's output with code B's model by the physical identity. The
//! second link is code B's own abstraction. A noiseless gadget composes exactly. With a
//! depolarising channel on one logical wire inside the gadget, the first link's residual is the
//! noise, the second link's stays zero, and the composite's measured residual sits under the law's
//! bound `‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂`, where `‖τ₂‖_post` is the Frobenius-induced norm of
//! code B's ideal recovery.
//!
//! This is an example with checks, not a theorem: it claims the residuals it measures and the
//! bound the law records.

mod constants;

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive, lift, lower};
use deep_causality_quantum::utils_tests::four_two_two;
use deep_causality_quantum::{
    CompositionLaw, LogicalGate, NumericCaps, code_switching, depolarizing_kraus,
};
use deep_causality_topology::LatticeComplex;

use crate::constants::{GADGET_NOISE, TORUS_SIDE};

/// The working type. Switch it to `f32` or `Float106`; nothing below changes.
pub type FloatType = f64;

/// The count word the logical bases are computed over.
pub type NumberType = u64;

/// The composite law of the switch `A → B` for `Z̄(0)`, with or without gadget noise.
fn law_for<S>(noise: Option<f64>) -> CompositionLaw<S>
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let code_a = four_two_two();
    let code_b = LatticeComplex::<2, S>::square_torus(TORUS_SIDE);
    let gadget = match noise {
        Some(p) => depolarizing_kraus::<S>(lift::<S>(p)).expect("a probability"),
        None => Vec::new(),
    };
    code_switching::<NumberType, _, _, S>(&code_a, &code_b, &LogicalGate::Z(0), gadget)
        .expect("codes with two logical qubits switch")
        .compose(&NumericCaps::default())
        .expect("both links have Io squares under the default caps")
        .law
}

fn report<S>(name: &str, noise: Option<f64>)
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let law = law_for::<S>(noise);
    let row = &law.rows[0];
    println!(
        "    {name:>8}: ε₁ = {:.3e}, ε₂ = {:.2e}, ‖τ₁‖_pre = {:.4}, ‖τ₂‖_post = {:.4}, bound = {:.3e}, measured = {:.3e}, {}",
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
    println!("=== QCL-2 chain: code switching [[4,2,2]] → [[8,2,2]] through a gadget ===\n");
    println!("[chain] L: encode A, decode A, (noise), encode B, run Z̄_B; M: code B's model; H: Z̄");
    println!("        first link: the gadget, aligned by the physical identity; second: code B\n");

    println!("[noiseless gadget] at FloatType");
    let clean = law_for::<FloatType>(None);
    print!("{clean}");
    assert!(
        lower(clean.measured()) < 1e-9,
        "a noiseless switch composes exactly"
    );
    println!("    exact: the gadget is the identity on the logical space\n");

    println!(
        "[gadget with depolarising noise p = {GADGET_NOISE} on one logical wire] at FloatType"
    );
    let noisy = law_for::<FloatType>(Some(GADGET_NOISE));
    print!("{noisy}");
    let row = &noisy.rows[0];
    assert!(lower(row.epsilon_first) > 0.0 && lower(row.epsilon_second) < 1e-9);
    println!(
        "    the noise is the first link's residual; code B's link stays exact; the composite \
         sits under ‖τ₂‖_post · ε₁ = {:.3e}\n",
        lower(row.post * row.epsilon_first)
    );

    println!("[noisy gadget] at the three shipped precisions");
    report::<f32>("f32", Some(GADGET_NOISE));
    report::<f64>("f64", Some(GADGET_NOISE));
    report::<Float106>("Float106", Some(GADGET_NOISE));
    println!("\n=== done: an example with checks, not a theorem ===");
}
