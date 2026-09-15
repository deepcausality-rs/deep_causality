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

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive, lift, lower};
use deep_causality_quantum::utils_tests::four_two_two;
use deep_causality_quantum::{CompositionLaw, NumericCaps, distillation_round};

use crate::constants::NOISE_SWEEP;

/// The working type. Switch it to `f32` or `Float106`; nothing below changes.
pub type FloatType = f64;

/// The count word the logical basis is computed over.
pub type NumberType = u64;

fn law_for<S>(p: f64) -> CompositionLaw<S>
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    distillation_round::<NumberType, _, S>(&four_two_two(), lift::<S>(p))
        .expect("a probability in [0, 1] on the [[4,2,2]] code")
        .compose(&NumericCaps::default())
        .expect("both links have Io squares under the default caps")
        .law
}

fn report<S>(name: &str, p: f64)
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let law = law_for::<S>(p);
    let row = &law.rows[0];
    println!(
        "    {name:>8}: p = {p}: ε₁ = {:.3e}, ε₂ = {:.2e}, ‖τ₂‖_post = {:.4}, bound = {:.3e}, measured = {:.3e}, {}",
        lower(row.epsilon_first),
        lower(row.epsilon_second),
        lower(row.post),
        lower(row.bound),
        lower(row.measured),
        if law.holds() { "holds" } else { "violated" }
    );
    assert!(law.holds(), "the composition law holds at every precision");
}

fn main() {
    println!("=== QCL-2 chain: a distillation round on the [[4,2,2]] code ===\n");
    println!(
        "[chain] L: encode, depolarise every qubit with p, run T̄ H̄; M: the same without noise;"
    );
    println!("        H: T H on two qubits. First link: the noise; second link: the code\n");

    for p in NOISE_SWEEP {
        let law = law_for::<FloatType>(p);
        println!("[p = {p}] at FloatType");
        print!("{law}");
        let row = &law.rows[0];
        if p == 0.0 {
            assert!(lower(row.measured) < 1e-9, "the noiseless round is exact");
            println!("    exact: without noise both links commute\n");
        } else {
            assert!(lower(row.epsilon_first) > 0.0 && lower(row.epsilon_second) < 1e-9);
            println!(
                "    the recovery leaves {:.3e} of the noise's {:.3e}; the law allows {:.3e}\n",
                lower(row.measured),
                lower(row.epsilon_first),
                lower(row.bound)
            );
        }
    }

    println!("[p = {}] at the three shipped precisions", NOISE_SWEEP[2]);
    report::<f32>("f32", NOISE_SWEEP[2]);
    report::<f64>("f64", NOISE_SWEEP[2]);
    report::<Float106>("Float106", NOISE_SWEEP[2]);
    println!(
        "\n=== done: an example with checks, not a theorem; the paper defers this case (§7.1) ==="
    );
}
