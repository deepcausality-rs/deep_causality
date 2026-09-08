/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared assertions for the numeric suites.
//!
//! Each states what it accepts and what it refuses, so a failure message says which contract
//! was broken rather than only which numbers differed.

use crate::errors::stats_error::{StatsError, StatsErrorEnum};
use deep_causality_algebra::Real;
use deep_causality_num::{FromPrimitive, ToPrimitive, lift};

/// Asserts `|got − want| ≤ rel_tol · max(|want|, 1)`.
///
/// The absolute floor of 1 keeps the comparison meaningful when `want` is zero; cases whose target
/// is zero or is smaller than one assert exact equality instead, so the floor never softens them.
/// `to_f64` appears only in the panic message.
pub fn assert_close<T: Real + FromPrimitive + ToPrimitive>(
    got: T,
    want: T,
    rel_tol: f64,
    what: &str,
) {
    let one = lift::<T>(1.0);
    let scale = if want.abs() > one { want.abs() } else { one };
    let tol = lift::<T>(rel_tol) * scale;
    assert!(
        (got - want).abs() <= tol,
        "{what}: got {:?}, want {:?} within {rel_tol:e} relative",
        got.to_f64(),
        want.to_f64()
    );
}

/// Asserts bit equality in the working scalar, for targets the type represents exactly.
pub fn assert_exact<T: Real + ToPrimitive>(got: T, want: T, what: &str) {
    assert!(
        got == want,
        "{what}: got {:?}, want exactly {:?}",
        got.to_f64(),
        want.to_f64()
    );
}

pub fn expect_empty_input<T>(got: Result<T, StatsError>, what: &str) {
    match got {
        Err(StatsError(StatsErrorEnum::EmptyInput(_))) => {}
        Err(StatsError(other)) => panic!("{what}: refused with {other:?}, expected EmptyInput"),
        Ok(_) => panic!("{what}: returned a value; the empty sample has no answer to return"),
    }
}

pub fn expect_insufficient_samples<T>(got: Result<T, StatsError>, what: &str) {
    match got {
        Err(StatsError(StatsErrorEnum::InsufficientSamples(_))) => {}
        Err(StatsError(other)) => {
            panic!("{what}: refused with {other:?}, expected InsufficientSamples")
        }
        Ok(_) => panic!(
            "{what}: returned a value. This is the behaviour change the crate exists to make: the \
             absorbed `variance_ddof1` returned `T::one()` for a sample shorter than two, and that \
             sentinel fed a Gaussian density. A typed error replaces it."
        ),
    }
}

/// Asserts that a sample carrying a non-finite entry yields no finite answer.
///
/// See reading 2 in the module header: the moments document no non-finite guard, so either an
/// `Err(NonFiniteInput)` or a non-finite `Ok` satisfies the contract, and only a finite number —
/// the signature of an implementation that quietly skipped the entry — fails it.
pub fn assert_no_finite_answer<T: Real + ToPrimitive>(got: Result<T, StatsError>, what: &str) {
    match got {
        Ok(v) => assert!(
            !v.is_finite(),
            "{what}: invented the finite value {:?} from a sample carrying a non-finite entry",
            v.to_f64()
        ),
        Err(StatsError(StatsErrorEnum::NonFiniteInput(_))) => {}
        Err(StatsError(other)) => panic!(
            "{what}: refused with {other:?}; the only refusal documented for a non-finite entry is \
             NonFiniteInput"
        ),
    }
}
