/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::solvers::dec::DecNsRate;
use crate::traits::FluidTheory;
use crate::types::{Ambient, CfdScalar};
use deep_causality_physics::{PhysicsError, VelocityOneForm};
use deep_causality_topology::HodgeDecomposeOptions;

/// The DEC-native incompressible Navier–Stokes regime as a [`FluidTheory`].
///
/// Wraps the validated [`DecNsRate`] together with the projection options the
/// rate's per-step CG solve needs, so the field-level rate matches the
/// `rate(state, ambient)` seam. The manifold borrow lives inside the rate (the
/// theory is materialized bound to the manifold at run time), keeping the
/// `FluidTheory` trait itself `'m`-free.
#[derive(Debug)]
pub struct DecIncompressible<'m, const D: usize, R: CfdScalar> {
    rate: DecNsRate<'m, D, R>,
    opts: HodgeDecomposeOptions<R>,
}

impl<'m, const D: usize, R: CfdScalar> DecIncompressible<'m, D, R> {
    /// Build the theory from a validated rate and the projection options.
    pub fn new(rate: DecNsRate<'m, D, R>, opts: HodgeDecomposeOptions<R>) -> Self {
        Self { rate, opts }
    }

    /// The underlying DEC rate.
    pub fn rate(&self) -> &DecNsRate<'m, D, R> {
        &self.rate
    }
}

impl<'m, const D: usize, R: CfdScalar> FluidTheory<R> for DecIncompressible<'m, D, R> {
    type State = VelocityOneForm<R>;
    type Ambient = Ambient<R>;

    /// `P(−i_u(du♭) − ν Δ_dR u♭ + g♭)` at the ambient `ν`. The ambient is read
    /// once per call (between `Rk4` steps); with a constant ambient this is
    /// bit-identical to the construction-fixed rate.
    fn rate(
        &self,
        state: &VelocityOneForm<R>,
        ambient: &Ambient<R>,
    ) -> Result<VelocityOneForm<R>, PhysicsError> {
        self.rate.set_nu(*ambient.nu());
        self.rate.eval_projected(state, &self.opts)
    }
}
