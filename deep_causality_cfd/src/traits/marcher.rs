/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CfdScalar;
use deep_causality_physics::PhysicsError;

/// One projected step of a CFD solver: the theory's integration (RK4) followed by
/// the Leray projection back into the divergence-free type-state and the CFL guard,
/// reading the [`crate::Ambient`] for that step.
///
/// This is the per-step advance value the CfdFlow march drives via the arrow-algebra
/// iterator (`iterate_until` / `iterate_n`), interleaving between-step coupling
/// stages between calls.
pub trait Marcher<R: CfdScalar> {
    /// The marching state advanced by one step.
    type State;

    /// The ambient read for this step.
    type Ambient;

    /// The per-step output (the advanced state plus the diagnostics the step
    /// already computed).
    type Output;

    /// Advance the state by one projected step under the given ambient.
    fn advance(
        &self,
        state: &Self::State,
        ambient: &Self::Ambient,
    ) -> Result<Self::Output, PhysicsError>;
}
