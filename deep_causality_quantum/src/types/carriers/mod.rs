/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The typed carriers: what a QCL stage holds instead of a hand-packed tensor.
//!
//! Four frictions in the design note's §5 were ergonomic rather than physical, and all four were
//! the same shape: a validated value abandoned to compute, then re-validated, with its invariant
//! living in the caller's head between the two. The carriers close them by the rule
//! `EnvironmentalPrep` already follows. **Seal the interior and expose the operations.** A carrier
//! exposes read accessors and operations returning a new carrier, never `&mut` to a field whose
//! invariant was established at construction, so every value a caller can observe has passed the
//! checks its type names.
//!
//! | Carrier | Replaces | Validated at construction |
//! |---|---|---|
//! | [`QubitOperator`] | a flat complex slice against a hand-computed stride | a `2 × 2` unitary |
//! | [`Channel`] | a bare Kraus slice read as a channel at the call site | CPTP, once |
//! | [`QuantumPlant`] | state and channel juggling | a density matrix, re-validated after each evolution |
//! | [`Observable`] | ket → projector → Born, written out each time | a rank-1 Hermitian idempotent |
//!
//! Every carrier is generic in its scalar and names no width. The bound is `R: RealField`, and
//! that bound comes from the carrier rather than from its operations: every impl for `Complex<T>`
//! in `deep_causality_num_complex` is written `impl<T: RealField>`, so a carrier holding
//! `Complex<R>` reaches no algebraic structure below it. A reader looking for a relaxation to
//! `Real` will not find one here, and should not look.

pub(crate) mod observable;
pub(crate) mod quantum_channel;
pub(crate) mod quantum_plant;
pub(crate) mod qubit_operator;

pub use observable::*;
pub use quantum_channel::*;
pub use quantum_plant::*;
pub use qubit_operator::*;
