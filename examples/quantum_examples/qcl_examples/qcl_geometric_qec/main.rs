/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The QCL code path on the toric code, verified exactly
//!
//! Kitaev's toric code is a lattice, read as a code. Put a qubit on every edge of a square torus,
//! take the faces as `Z` checks and the vertices as `X` checks, and the number of logical qubits
//! you get is `β₁`, the first Betti number — the count of independent loops the surface has. The
//! code's structure is the surface's topology, and nothing else.
//!
//! That is what makes this example exact. `validate` runs four checks over the chain complex:
//!
//! ```text
//! derive_code              n from the 1-cells, k from β₁ over 𝔽₂,
//!                          Z checks from the columns of ∂₂, X checks from the columns of δ₀
//! check_ldpc_weights       both weights of both check matrices, against a declared bound
//! check_class_invariance   Z̄, S̄ and T̄ act on the homology class, not the representative
//! check_clifford_action    H̄ swaps the logical Paulis, by a symplectic tableau
//! ```
//!
//! Every verdict is an 𝔽₂ or rational computation over supports. Nothing here is simulated, and
//! nothing could be: the in-process simulator caps at 24 qubits and this code has 32, so a
//! state-vector check could not reach it. The exact predicates do not need one.
//!
//! # A check that rejects
//!
//! The run checks the LDPC weights twice: once against a bound the code meets and once against one
//! it does not. A validation suite that only ever accepts says nothing about whether it would
//! notice, so the second bound is there to make the first one worth reading, and the rejection
//! names the offending generator and its margin.

mod constants;
mod model;
mod utils_print;

use deep_causality_homology::ChainComplex;
use deep_causality_num::Float106;
use deep_causality_quantum::{CheckVerdict, QclBuilder, check_ldpc_weights, derive_code};

use crate::constants::{LDPC_BOUND, LDPC_BOUND_TOO_TIGHT};
use crate::model::square_torus;
use utils_print::{
    print_complex, print_derived_code, print_header, print_outcome, print_rejection, print_stages,
    print_structural_checks, print_weights,
};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`.
///
/// It carries less here than in most of these examples, and that is the point: every check on the
/// code is exact over 𝔽₂, so the scalar reaches only the lattice's coordinates and the margins the
/// reports record. A verdict that moved when the alias moved would mean a predicate had stopped
/// being exact.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

/// The count working type.
pub type NumberType = u64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let complex = square_torus();
    print_complex(
        complex.num_cells(0),
        complex.num_cells(1),
        complex.num_cells(2),
    );

    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_code(complex.clone())
        .build()?;

    let screened = QclBuilder::validate(&cfg)
        .derive_code()
        .check_ldpc_weights(LDPC_BOUND)
        .check_class_invariance()
        .check_clifford_action()
        .finalize()?;

    print_stages(&screened);

    // What `derive_code` read off the complex.
    let code = derive_code::<NumberType, _>(&complex)?;
    print_derived_code(&code);

    // Both weights against a bound the code meets.
    let met = check_ldpc_weights::<FloatType, NumberType>(&code, LDPC_BOUND)?;
    print_weights(LDPC_BOUND, &met);

    // And against one it does not, so the run shows the check rejecting as well as accepting.
    let too_tight = check_ldpc_weights::<FloatType, NumberType>(&code, LDPC_BOUND_TOO_TIGHT)?;
    let rejection = too_tight
        .report
        .first_rejection()
        .ok_or("a bound below the code's weight should have rejected")?;
    let offender = too_tight
        .offender
        .ok_or("a rejection should name the generator it rejected")?;

    print_rejection(LDPC_BOUND_TOO_TIGHT, offender, rejection.margin, &too_tight);

    print_structural_checks(&screened, code.z_generators().len());

    let verdict = screened
        .report()
        .ok_or("the screen should carry a current report")?
        .verdict();

    print_outcome(verdict, verdict == CheckVerdict::Accepted);

    if verdict != CheckVerdict::Accepted {
        return Err(
            format!("the screen returned {verdict:?}; every exact check should accept").into(),
        );
    }

    Ok(())
}
