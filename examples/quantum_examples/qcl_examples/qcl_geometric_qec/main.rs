/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The QCL code path on the toric code.
//!
//! `validate` takes a chain complex and runs four exact checks over it:
//!
//!   * `derive_code`: `n` from the 1-cells, `k` from `β₁` over 𝔽₂, the Z checks from the columns
//!     of `∂₂` and the X checks from the columns of `δ₀`;
//!   * `check_ldpc_weights`: both weights of both check matrices against a declared bound;
//!   * `check_class_invariance`: `Z̄`, `S̄` and `T̄` act on the homology class rather than on the
//!     representative, decided over the code space (Haruna, arXiv:2511.15224, Eq. 3.20);
//!   * `check_clifford_action`: `H̄` swaps the logical Paulis, decided by a symplectic tableau.
//!
//! Every verdict is an 𝔽₂ or rational computation over supports. Nothing here is simulated: the
//! in-process simulator caps at 24 qubits and this code has 32, so a state-vector check could not
//! reach it, and the exact predicates do not need one.

mod constants;
mod model;

use deep_causality_homology::ChainComplex;
use deep_causality_quantum::{CheckVerdict, QclBuilder, check_ldpc_weights, derive_code};

use crate::constants::{LDPC_BOUND, LDPC_BOUND_TOO_TIGHT};
use crate::model::square_torus;

/// The real working type; only the lattice complex's coordinates and the report margins carry it,
/// since every check on the code is exact over 𝔽₂.
pub type FloatType = f64;

/// The count working type.
pub type NumberType = u64;

fn main() {
    println!("=== QCL code path: the [[32, 2]] toric code, verified exactly ===\n");

    let complex = square_torus();
    println!(
        "[subject] square torus, {} vertices, {} edges, {} faces",
        complex.num_cells(0),
        complex.num_cells(1),
        complex.num_cells(2)
    );

    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_code(complex.clone())
        .build()
        .expect("a complex with 1-cells builds");
    println!(
        "    no probes, no baseline, no evidence: the code subject offers validate stages only\n"
    );

    let screened = QclBuilder::validate(&cfg)
        .derive_code()
        .check_ldpc_weights(LDPC_BOUND)
        .check_class_invariance()
        .check_clifford_action()
        .finalize()
        .expect("the toric code passes every exact check");

    println!("[validate]");
    for (name, report) in screened.stages() {
        println!(
            "    {name:<24} {:?}  examined {}",
            report.verdict(),
            report.examined()
        );
    }

    // What derive_code read off the complex.
    let code = derive_code::<u64, _>(&complex).expect("the code derives");
    println!("\n[derive_code]  [[n = {}, k = {}]]", code.n(), code.k());
    println!(
        "    {} Z checks of weight {}, {} X checks of weight {}, no distance claimed",
        code.z_generators().len(),
        code.z_generators()[0].weight(),
        code.x_generators().len(),
        code.x_generators()[0].weight()
    );

    // Both weights against the bound, and against one too tight for it.
    let w = check_ldpc_weights::<FloatType, u64>(&code, LDPC_BOUND).expect("weights");
    println!(
        "\n[check_ldpc_weights]  bound {LDPC_BOUND}: max column weight {}, max row weight {}, {} items examined, {:?}",
        w.max_column_weight,
        w.max_row_weight,
        w.report.examined(),
        w.report.verdict()
    );
    let tight = check_ldpc_weights::<FloatType, u64>(&code, LDPC_BOUND_TOO_TIGHT).expect("weights");
    let rejecting = tight
        .report
        .first_rejection()
        .expect("a bound of 3 rejects");
    println!(
        "    bound {LDPC_BOUND_TOO_TIGHT}: rejected at {:?} with margin {:.3} after {} items",
        tight.offender.expect("named"),
        rejecting.margin,
        tight.report.examined()
    );

    println!(
        "\n[check_class_invariance]  {} (class, gate) pairs over {} boundaries each: Z̄, S̄, T̄ act on the class",
        screened.stages()[2].1.examined(),
        code.z_generators().len()
    );
    println!(
        "[check_clifford_action]   {} logical Hadamards swap Z̄(γ) ↔ X̄(γ̃), up to phase and stabilizers",
        screened.stages()[3].1.examined()
    );
    println!(
        "\nscreen: {:?}. Verified by exact 𝔽₂ predicates; not simulated. SimQpu caps below this code's width.",
        screened.report().expect("current").verdict()
    );
    assert_eq!(
        screened.report().expect("current").verdict(),
        CheckVerdict::Accepted
    );
}
