/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CfdScalar;
use crate::types::flow::Report;
use deep_causality_physics::PhysicsError;

/// The shared seam of the three CfdFlow solver kinds — the marching solver, the
/// MMS-verification solver, and the operator-accuracy solver. Each consumes its
/// fully-owned case (materializing any borrows as locals) and yields a common
/// [`Report`]. Adding a fourth kind is an implementation of this trait, not a
/// change to the DSL core (design D2).
pub trait Solver<R: CfdScalar> {
    /// Run the case to completion and return its owned report.
    ///
    /// # Errors
    /// Any failure of materialization, marching, kernel evaluation, or operator
    /// sweep — surfaced as a [`PhysicsError`].
    fn run(self) -> Result<Report<R>, PhysicsError>;
}
