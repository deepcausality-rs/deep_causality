/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::qgates::{gates_haruna, mechanics};
use crate::types::qpu::circuit::{GateOp, LogicalProgram};
use crate::{Gate, Operator};
use alloc::vec::Vec;
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_homology::Gf2Chain;
use deep_causality_multivector::HilbertState;
use deep_causality_num::{FromPrimitive, NaturalNumber};

/// Causal wrapper for [`mechanics::born_probability_kernel`].
///
/// The kernel validates and clamps the probability to `[0, 1]`, so the value
/// channel carries the plain real directly.
pub fn born_probability<R>(state: &HilbertState<R>, basis: &HilbertState<R>) -> PropagatingEffect<R>
where
    R: RealField + FromPrimitive + core::iter::Sum + Default + Debug,
{
    match mechanics::born_probability_kernel(state, basis) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::expectation_value_kernel`].
pub fn expectation_value<R>(state: &HilbertState<R>, operator: &Operator<R>) -> PropagatingEffect<R>
where
    R: RealField + FromPrimitive + core::iter::Sum + Default + Debug,
{
    match mechanics::expectation_value_kernel(state, operator) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::apply_gate_kernel`].
pub fn apply_gate<R>(state: &HilbertState<R>, gate: &Gate<R>) -> PropagatingEffect<HilbertState<R>>
where
    R: RealField + Debug,
{
    match mechanics::apply_gate_kernel(state, gate) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::commutator_kernel`].
pub fn commutator<R>(a: &Operator<R>, b: &Operator<R>) -> PropagatingEffect<HilbertState<R>>
where
    R: RealField + Debug,
{
    match mechanics::commutator_kernel(a, b) {
        Ok(res) => PropagatingEffect::pure(res),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ---------------------------------------------------------------------------
// Haruna logical gates on the causal monad.
//
// These wrap `gates_haruna` rather than a `mechanics` kernel: the gate builders
// are pure combinatorics over a chain's support and have no numeric kernel to
// adapt. The value carried is the physical-gate program, `Vec<GateOp>`, which is
// what Table 1's second column produces.
// ---------------------------------------------------------------------------

/// Causal wrapper for [`gates_haruna::logical_z`].
pub fn haruna_z_gate<W: NaturalNumber>(gamma: &Gf2Chain<W>) -> PropagatingEffect<Vec<GateOp>> {
    PropagatingEffect::pure(gates_haruna::logical_z(gamma))
}

/// Causal wrapper for [`gates_haruna::logical_x`].
pub fn haruna_x_gate<W: NaturalNumber>(
    gamma_tilde: &Gf2Chain<W>,
) -> PropagatingEffect<Vec<GateOp>> {
    PropagatingEffect::pure(gates_haruna::logical_x(gamma_tilde))
}

/// Causal wrapper for [`gates_haruna::logical_s`].
pub fn haruna_s_gate<W: NaturalNumber>(gamma: &Gf2Chain<W>) -> PropagatingEffect<Vec<GateOp>> {
    PropagatingEffect::pure(gates_haruna::logical_s(gamma))
}

/// Causal wrapper for [`gates_haruna::logical_t`].
///
/// Fallible since the tuple cap: a chain whose weight puts `C(w, 3)` above
/// [`TUPLE_ENUMERATION_CAP`](gates_haruna::TUPLE_ENUMERATION_CAP) reaches
/// `from_error` rather than allocating.
pub fn haruna_t_gate<W: NaturalNumber>(gamma: &Gf2Chain<W>) -> PropagatingEffect<Vec<GateOp>> {
    match gates_haruna::logical_t(gamma) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`gates_haruna::logical_cz`].
pub fn haruna_cz_gate<W: NaturalNumber>(
    gamma1: &Gf2Chain<W>,
    gamma2: &Gf2Chain<W>,
) -> PropagatingEffect<Vec<GateOp>> {
    match gates_haruna::logical_cz(gamma1, gamma2) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`gates_haruna::logical_hadamard`].
///
/// The value carried is a [`LogicalProgram`], the gate program together with
/// the `e^{-iπ/4}` global phase Table 1 attaches. An earlier version of this
/// wrapper dropped the phase, on the reasoning that a `PropagatingEffect`
/// carries one value and the circuit is the one a caller runs. That foreclosed
/// the exact form of the Clifford check and any controlled use of the gate, so
/// the phase now travels with the program and the caller decides.
pub fn haruna_hadamard_gate<W: NaturalNumber, R>(
    gamma: &Gf2Chain<W>,
    gamma_tilde: &Gf2Chain<W>,
) -> PropagatingEffect<LogicalProgram<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match gates_haruna::logical_hadamard::<W, R>(gamma, gamma_tilde) {
        Ok((ops, phase)) => PropagatingEffect::pure(LogicalProgram::with_global_phase(ops, phase)),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::fidelity_kernel`].
///
/// Like [`born_probability`], the value channel carries the plain real
/// fidelity in `[0, 1]`.
pub fn fidelity<R>(ideal: &HilbertState<R>, actual: &HilbertState<R>) -> PropagatingEffect<R>
where
    R: RealField + FromPrimitive + core::iter::Sum + Default + Debug,
{
    match mechanics::fidelity_kernel(ideal, actual) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
