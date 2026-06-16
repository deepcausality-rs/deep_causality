/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `MarchConfig` — the owned, reusable **configuration container** for a DEC incompressible
//! marching case. Pure data (no manifold borrow, design D2): it bundles the scenario (mesh, seed,
//! observe), the solver config, the boundary-zone tuple, and the optional between-step coupling.
//! Built by [`CfdConfigBuilder::march`](crate::CfdConfigBuilder); composed and run by the
//! [`CfdFlow`](crate::CfdFlow) DSL, which borrows a caller-owned geometry (B1) — see
//! [`MarchConfig::materialize`].

use crate::solvers::DecNsConfig;
use crate::solvers::dec::BoundaryZone;
use crate::types::CfdScalar;
use crate::types::flow::PhysicsStage;
use crate::types::flow_config::{Mesh, Observe, Seed};
use deep_causality_physics::PhysicsError;
use deep_causality_topology::{LatticeComplex, Manifold};

/// When the march stops.
#[derive(Debug, Clone, Copy)]
pub enum MarchStop<R: CfdScalar> {
    /// March a fixed number of steps.
    Fixed(usize),
    /// March until the step-to-step kinetic-energy change drops below `tol`
    /// (steady state), or `max_steps` is reached.
    Steady { tol: R, max_steps: usize },
}

/// The owned configuration container for a marching case. Holds only owned specs; the same config
/// can be materialized and run repeatedly (factual + counterfactual). The boundary-zone tuple `Z`
/// and the coupling `C` compose statically (each `()` by default).
pub struct MarchConfig<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>, C: PhysicsStage<D, R>> {
    pub(crate) name: String,
    pub(crate) mesh: Mesh<D, R>,
    pub(crate) solver: DecNsConfig<R>,
    /// A prescribed moving wall as `(axis, max_side, velocity)`, applied via the solver's
    /// `with_moving_wall` builder (reuses the existing mechanism; no `MovingWall` type collision).
    pub(crate) moving_wall: Option<(usize, bool, [R; D])>,
    pub(crate) seed: Seed,
    pub(crate) stop: MarchStop<R>,
    pub(crate) observe: Observe<D, R>,
    pub(crate) zones: Z,
    /// The between-step multi-physics coupling (`()` for single-physics).
    pub(crate) coupling: C,
    /// Uniform initial coupled scalar fields, sized to the cell count at materialization.
    pub(crate) coupled_scalars: Vec<(String, R)>,
}

impl<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>, C: PhysicsStage<D, R>>
    MarchConfig<D, R, Z, C>
{
    /// Assemble a marching config from its owned specs (constructed by the config builder).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        name: impl Into<String>,
        mesh: Mesh<D, R>,
        solver: DecNsConfig<R>,
        moving_wall: Option<(usize, bool, [R; D])>,
        seed: Seed,
        stop: MarchStop<R>,
        observe: Observe<D, R>,
        zones: Z,
        coupling: C,
        coupled_scalars: Vec<(String, R)>,
    ) -> Self {
        Self {
            name: name.into(),
            mesh,
            solver,
            moving_wall,
            seed,
            stop,
            observe,
            zones,
            coupling,
            coupled_scalars,
        }
    }

    /// Materialize the metric-bearing lattice manifold — **the geometry the caller owns** and lends
    /// to the [`CfdFlow`](crate::CfdFlow) pipeline via `.on(&manifold)` (B1). A proven config transposes
    /// to another geometry by materializing a different mesh, and an expensive geometry is
    /// materialized once and run many times.
    ///
    /// # Errors
    /// Any failure building the lattice, metric, or cut-cell geometry.
    pub fn materialize(&self) -> Result<Manifold<LatticeComplex<D, R>, R>, PhysicsError> {
        self.mesh.materialize().map(|(manifold, _registry)| manifold)
    }
}
