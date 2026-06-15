/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::solvers::DecNsConfig;
use crate::solvers::dec::diagnostics::dec_kinetic_energy;
use crate::types::CfdScalar;
use crate::types::flow::{Mesh, Observe, Report, Seed};
use deep_causality_physics::PhysicsError;

/// An owned, runnable marching case for the DEC incompressible solver. Holds only
/// owned specs (no manifold borrow); `run` materializes the manifold + solver as
/// locals, marches, and returns an owned `Report` (design D2).
///
/// This is the first of the three solver kinds (the others — MMS verification and
/// operator-accuracy — are added next). Boundary zones, the fluent `Flow::march`
/// builder, the uncertain causal-monad march, and `.couple` layer on from here.
#[derive(Debug, Clone)]
pub struct MarchCase<const D: usize, R: CfdScalar> {
    name: String,
    mesh: Mesh<D, R>,
    solver: DecNsConfig<R>,
    /// A prescribed moving wall as `(axis, max_side, velocity)`, applied via the
    /// solver's `with_moving_wall` builder. Reuses the existing solver mechanism, so
    /// no type collides with the `MovingWall` boundary zone.
    moving_wall: Option<(usize, bool, [R; D])>,
    seed: Seed,
    steps: usize,
    observe: Observe,
}

impl<const D: usize, R: CfdScalar> MarchCase<D, R> {
    /// Assemble a marching case from its owned specs (constructed by `Flow::march`).
    pub(crate) fn new(
        name: impl Into<String>,
        mesh: Mesh<D, R>,
        solver: DecNsConfig<R>,
        moving_wall: Option<(usize, bool, [R; D])>,
        seed: Seed,
        steps: usize,
        observe: Observe,
    ) -> Self {
        Self {
            name: name.into(),
            mesh,
            solver,
            moving_wall,
            seed,
            steps,
            observe,
        }
    }

    /// Materialize the domain, seed, march `steps` projected steps, and collect the
    /// observed series. Borrows (manifold, solver) stay inside this call.
    pub fn run(&self) -> Result<Report<R>, PhysicsError> {
        let manifold = self.mesh.materialize()?;
        let solver = self.solver.materialize_with_zones(&manifold, ())?;
        let solver = match self.moving_wall {
            Some((axis, max_side, velocity)) => {
                solver.with_moving_wall(axis, max_side, velocity)?
            }
            None => solver,
        };
        let mut state = self.seed.apply(&solver, &manifold)?;

        let mut energy: Vec<R> = Vec::with_capacity(self.steps + 1);
        if self.observe.kinetic_energy {
            energy.push(dec_kinetic_energy(&manifold, state.as_one_form())?);
        }
        for _ in 0..self.steps {
            let output = solver.step(&state)?;
            state = output.into_state();
            if self.observe.kinetic_energy {
                energy.push(dec_kinetic_energy(&manifold, state.as_one_form())?);
            }
        }

        let mut report = Report::new(self.name.clone());
        if self.observe.kinetic_energy {
            report.add_series("kinetic_energy", energy);
        }
        Ok(report)
    }
}
