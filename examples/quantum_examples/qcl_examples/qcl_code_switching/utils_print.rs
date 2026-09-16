/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the code-switching example.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::constants::{GADGET_NOISE_LABEL, TORUS_SIDE};
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lower};
use deep_causality_quantum::CompositionLaw;

pub fn print_header() {
    println!("=== QCL-2 chain: code switching [[4,2,2]] -> [[8,2,2]] through a gadget ===\n");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Chain      L: encode A, decode A, (noise), encode B, run Z-bar_B");
    println!("           M: code B's model");
    println!("           H: Z-bar");
    println!("Links      first: the gadget, aligned by the physical identity.  second: code B");
    println!("Code B     the [[8,2,2]] toric code on a {TORUS_SIDE}x{TORUS_SIDE} square torus\n");
}

/// The noiseless switch.
pub fn print_clean(law: &CompositionLaw<FloatType>, exact: bool) {
    println!("[noiseless gadget] at FloatType");
    print!("{law}");

    if exact {
        println!("    exact: the gadget is the identity on the logical space");
    } else {
        println!("    NOT exact: a residual survived a gadget that is the logical identity");
    }
    println!();
}

/// The noisy switch.
pub fn print_noisy(law: &CompositionLaw<FloatType>, second_link_exact: bool) {
    let row = &law.rows[0];

    println!(
        "[gadget with depolarising noise p = {GADGET_NOISE_LABEL} on one logical wire] at FloatType"
    );
    print!("{law}");
    println!(
        "    the noise is the first link's residual; code B's link stays {}; the composite",
        if second_link_exact {
            "exact"
        } else {
            "INEXACT"
        }
    );
    println!(
        "    sits under |t2|_post * e1 = {:.3e}",
        lower(row.post * row.epsilon_first)
    );
    println!();
}

/// One precision's row.
pub fn print_row<S>(name: &str, law: &CompositionLaw<S>)
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let row = &law.rows[0];

    println!(
        "    {name:>8}: e1 = {:.3e}, e2 = {:.2e}, |t1|_pre = {:.4}, |t2|_post = {:.4}, bound = {:.3e}, measured = {:.3e}, {}",
        lower(row.epsilon_first),
        lower(row.epsilon_second),
        lower(row.pre),
        lower(row.post),
        lower(row.bound),
        lower(row.measured),
        if law.holds() { "holds" } else { "VIOLATED" }
    );
}

/// What the run came to.
pub fn print_outcome(every_law_holds: bool, clean_is_exact: bool) {
    println!();
    println!("Outcome");
    println!(
        "  every composition law held    {}",
        yes_no(every_law_holds)
    );
    println!("  the noiseless switch exact    {}", yes_no(clean_is_exact));
    println!();
    println!("  Switching codes is how a machine gets gates one code cannot do transversally.");
    println!("  What it costs is a gadget, and what the gadget costs is the first link's");
    println!("  residual. The law turns that into a bound on the whole switch, so the price of");
    println!("  the detour is a number rather than a worry.");
    println!();
    println!("  This is an example with checks, not a theorem.");
}

fn yes_no(ok: bool) -> &'static str {
    if ok { "yes" } else { "NO" }
}
