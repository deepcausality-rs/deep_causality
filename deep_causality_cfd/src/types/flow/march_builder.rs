/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::solvers::DecNsConfig;
use crate::solvers::dec::BoundaryZone;
use crate::types::CfdScalar;
use crate::types::flow::march_case::MarchStop;
use crate::types::flow::{MarchCase, Mesh, Observe, Report, Seed};
use deep_causality_physics::PhysicsError;

/// The **Flow** DSL entry point.
pub struct Flow;

impl Flow {
    /// Begin a marching case (DEC incompressible). The mesh pins the dimension `D`
    /// and precision `R`; boundary zones default to `()` (a closed/periodic domain)
    /// until `.zones(...)` is called.
    pub fn march<const D: usize, R: CfdScalar>(name: impl Into<String>) -> MarchBuilder<D, R, ()> {
        MarchBuilder::new(name)
    }
}

/// Fluent builder for a marching case. Accumulates owned specs; `run` assembles the
/// [`MarchCase`] and executes it. The boundary-zone tuple `Z` (default `()`) is set
/// via [`MarchBuilder::zones`], which transitions the builder type.
pub struct MarchBuilder<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>> {
    name: String,
    mesh: Option<Mesh<D, R>>,
    solver: Option<DecNsConfig<R>>,
    moving_wall: Option<(usize, bool, [R; D])>,
    seed: Seed,
    stop: MarchStop<R>,
    observe: Observe<D, R>,
    zones: Z,
}

impl<const D: usize, R: CfdScalar> MarchBuilder<D, R, ()> {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mesh: None,
            solver: None,
            moving_wall: None,
            seed: Seed::Rest,
            stop: MarchStop::Fixed(0),
            observe: Observe::default(),
            zones: (),
        }
    }
}

impl<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>> MarchBuilder<D, R, Z> {
    /// Set the composable boundary-zone tuple (e.g.
    /// `(Inflow, (Outflow, (SlipWall, SlipWall)))`). Transitions the builder type.
    pub fn zones<Z2: BoundaryZone<D, R>>(self, zones: Z2) -> MarchBuilder<D, R, Z2> {
        MarchBuilder {
            name: self.name,
            mesh: self.mesh,
            solver: self.solver,
            moving_wall: self.moving_wall,
            seed: self.seed,
            stop: self.stop,
            observe: self.observe,
            zones,
        }
    }

    /// The mesh (required).
    pub fn mesh(mut self, mesh: Mesh<D, R>) -> Self {
        self.mesh = Some(mesh);
        self
    }

    /// The DEC solver configuration (required).
    pub fn solver(mut self, config: DecNsConfig<R>) -> Self {
        self.solver = Some(config);
        self
    }

    /// A prescribed moving wall on `axis` (the far face when `max_side`) carrying
    /// the tangential `velocity` — applied via the solver's `with_moving_wall`.
    pub fn moving_wall(mut self, axis: usize, max_side: bool, velocity: [R; D]) -> Self {
        self.moving_wall = Some((axis, max_side, velocity));
        self
    }

    /// Convenience: drive the y-max face tangentially — the lid-driven cavity lid.
    pub fn lid(self, velocity: [R; D]) -> Self {
        self.moving_wall(1, true, velocity)
    }

    /// The initial condition (default `Seed::Rest`).
    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = seed;
        self
    }

    /// March a fixed number of steps.
    pub fn march_for(mut self, steps: usize) -> Self {
        self.stop = MarchStop::Fixed(steps);
        self
    }

    /// March until the step-to-step kinetic-energy change drops below `tol` (steady
    /// state), or `max_steps` is reached.
    pub fn march_until_steady(mut self, tol: R, max_steps: usize) -> Self {
        self.stop = MarchStop::Steady { tol, max_steps };
        self
    }

    /// The diagnostics to collect.
    pub fn observe(mut self, observe: Observe<D, R>) -> Self {
        self.observe = observe;
        self
    }

    /// Assemble and run the case.
    ///
    /// # Errors
    /// `PhysicsError::DimensionMismatch` when the mesh is missing,
    /// `PhysicsError::PhysicalInvariantBroken` when the solver config is missing,
    /// and any error from materialization or the march.
    pub fn run(self) -> Result<Report<R>, PhysicsError> {
        let mesh = self.mesh.ok_or_else(|| {
            PhysicsError::DimensionMismatch("Flow::march: a mesh is required".into())
        })?;
        let solver = self.solver.ok_or_else(|| {
            PhysicsError::PhysicalInvariantBroken("Flow::march: a solver config is required".into())
        })?;
        MarchCase::new(
            self.name,
            mesh,
            solver,
            self.moving_wall,
            self.seed,
            self.stop,
            self.observe,
            self.zones,
        )
        .run()
    }
}
