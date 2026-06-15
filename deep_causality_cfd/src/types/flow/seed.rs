/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::solvers::dec::DecNsSolver;
use crate::types::CfdScalar;
use deep_causality_physics::{PhysicsError, SolenoidalField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, LatticeComplex, Manifold};

/// A named initial condition. Static (no boxed closures) so a `Mesh`/case stays
/// `Clone`. The seed builds the vertex vector field and seeds it through the
/// solver's divergence-free projection.
#[derive(Debug, Clone, Copy)]
pub enum Seed {
    /// Fluid at rest (zero velocity).
    Rest,
    /// The classic 3D Taylor–Green vortex. Use with a fully periodic cube (`D == 3`).
    TaylorGreenVortex,
}

impl Seed {
    /// Build the initial divergence-free field for this seed.
    pub(crate) fn apply<const D: usize, R: CfdScalar>(
        &self,
        solver: &DecNsSolver<'_, D, R>,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
    ) -> Result<SolenoidalField<R>, PhysicsError> {
        let n0 = manifold.complex().num_cells(0);
        let vertex = match self {
            Seed::Rest => vec![R::zero(); D * n0],
            Seed::TaylorGreenVortex => taylor_green_vertex_field::<D, R>(manifold, n0),
        };
        let tensor = CausalTensor::new(vertex, vec![D * n0])
            .map_err(|e| PhysicsError::DimensionMismatch(format!("seed tensor: {e}")))?;
        solver.seed_from_vertex_vectors(&tensor)
    }
}

/// `u = sin(kx)cos(ky)cos(kz)`, `v = -cos(kx)sin(ky)cos(kz)`, `w = 0`, `k = 2π/n`.
fn taylor_green_vertex_field<const D: usize, R: CfdScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    n0: usize,
) -> Vec<R> {
    let lift = |x: f64| R::from_f64(x).expect("specification lifts into R");
    let n = manifold.complex().shape()[0];
    let k = lift(2.0 * core::f64::consts::PI) / lift(n as f64);
    let mut v = vec![R::zero(); D * n0];
    for (vi, cell) in manifold.complex().iter_cells(0).enumerate() {
        let p = cell.position();
        let (x, y, z) = (
            k * lift(p[0] as f64),
            k * lift(p[1] as f64),
            k * lift(p[2] as f64),
        );
        v[D * vi] = x.sin() * y.cos() * z.cos();
        v[D * vi + 1] = R::zero() - x.cos() * y.sin() * z.cos();
        // w-component stays zero.
    }
    v
}
