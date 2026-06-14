/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The body-force boundary zone: a forcing term added to the rate right-hand side.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{LatticeComplex, Manifold};

use crate::theories::fluid_dynamics::dec::DecNsScalar;

use super::boundary_zone::BoundaryZone;

/// A body force on the velocity edges: the edge-integral cochain `g♭` (e.g. a streamwise pressure
/// gradient `G·h` on the x-edges) added to the rate source. The carried tensor is the grade-1
/// edge cochain; the solver validates and wraps it as a `BodyForceOneForm` when assembling.
#[derive(Debug, Clone)]
pub struct BodyForceZone<R: DecNsScalar> {
    force: CausalTensor<R>,
}

impl<R: DecNsScalar> BodyForceZone<R> {
    /// A body force from its grade-1 edge-integral cochain.
    pub fn new(force: CausalTensor<R>) -> Self {
        Self { force }
    }

    /// The edge-integral cochain.
    pub fn force(&self) -> &CausalTensor<R> {
        &self.force
    }
}

impl<const D: usize, R: DecNsScalar> BoundaryZone<D, R> for BodyForceZone<R> {
    fn collect_rate_source(&self, _manifold: &Manifold<LatticeComplex<D, R>, R>, acc: &mut [R]) {
        for (a, f) in acc.iter_mut().zip(self.force.as_slice().iter()) {
            *a += *f;
        }
    }
}
