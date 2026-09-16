/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the distillation round.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, lower};
use deep_causality_quantum::CompositionLaw;

pub fn print_header() {
    println!("=== QCL-2 chain: a distillation round on the [[4,2,2]] code ===\n");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Chain      L: encode, depolarise every qubit with p, run T-bar H-bar");
    println!("           M: the same without the noise");
    println!("           H: T H on two qubits");
    println!("Links      first: the noise.  second: the code's abstraction\n");
}

/// One probability's round, with the law it produced.
pub fn print_round(label: &str, law: &CompositionLaw<FloatType>) {
    let row = &law.rows[0];

    println!("[p = {label}]");
    print!("{law}");

    if lower(row.epsilon_first) == 0.0 {
        println!("    exact: without noise both links commute\n");
        return;
    }

    println!(
        "    the recovery leaves {:.3e} of the noise's {:.3e}; the law allows {:.3e}\n",
        lower(row.measured),
        lower(row.epsilon_first),
        lower(row.bound)
    );
}

/// One precision's row of the comparison.
pub fn print_row<S>(name: &str, law: &CompositionLaw<S>)
where
    S: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let row = &law.rows[0];

    println!(
        "    {name:>8}: e1 = {:.3e}, e2 = {:.2e}, |t2|_post = {:.4}, bound = {:.3e}, measured = {:.3e}, {}",
        lower(row.epsilon_first),
        lower(row.epsilon_second),
        lower(row.post),
        lower(row.bound),
        lower(row.measured),
        if law.holds() { "holds" } else { "VIOLATED" }
    );
}

/// What the run came to.
pub fn print_outcome(every_law_holds: bool) {
    println!();

    if every_law_holds {
        println!("Every law held, at every probability and every precision.");
    } else {
        println!("A law was violated. The rows above name which.");
    }

    println!();
    println!("This is an example with checks, not a theorem: it claims the residuals it measures");
    println!("and the bound the law records. The paper defers this case (Lorenz & Tull, §7.1).");
}
