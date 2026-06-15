/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::solvers::DecNsConfig;
use crate::solvers::dec::BoundaryZone;
use crate::solvers::dec::diagnostics::{
    dec_divergence_residual, dec_kinetic_energy, dec_max_speed,
};
use crate::types::CfdScalar;
use crate::types::flow::{Mesh, Observe, Report, Seed};
use deep_causality_physics::{PhysicsError, SolenoidalField};
use deep_causality_topology::{LatticeComplex, Manifold};

/// When the march stops.
#[derive(Debug, Clone, Copy)]
pub(crate) enum MarchStop<R: CfdScalar> {
    /// March a fixed number of steps.
    Fixed(usize),
    /// March until the step-to-step kinetic-energy change drops below `tol`
    /// (steady state), or `max_steps` is reached.
    Steady { tol: R, max_steps: usize },
}

/// An owned, runnable marching case for the DEC incompressible solver. Holds only
/// owned specs (no manifold borrow); `run` materializes the manifold + solver as
/// locals, marches, and returns an owned `Report` (design D2). The boundary-zone
/// tuple `Z` composes statically (default `()` for a closed or periodic domain).
pub struct MarchCase<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>> {
    name: String,
    mesh: Mesh<D, R>,
    solver: DecNsConfig<R>,
    /// A prescribed moving wall as `(axis, max_side, velocity)`, applied via the
    /// solver's `with_moving_wall` builder. Reuses the existing solver mechanism, so
    /// no type collides with the `MovingWall` boundary zone.
    moving_wall: Option<(usize, bool, [R; D])>,
    seed: Seed,
    stop: MarchStop<R>,
    observe: Observe,
    zones: Z,
}

impl<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>> MarchCase<D, R, Z> {
    /// Assemble a marching case from its owned specs (constructed by `Flow::march`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        name: impl Into<String>,
        mesh: Mesh<D, R>,
        solver: DecNsConfig<R>,
        moving_wall: Option<(usize, bool, [R; D])>,
        seed: Seed,
        stop: MarchStop<R>,
        observe: Observe,
        zones: Z,
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
        }
    }

    /// Materialize the domain (with its boundary zones), seed, march, and collect the
    /// observed series. Borrows (manifold, solver) stay inside this call.
    pub fn run(self) -> Result<Report<R>, PhysicsError> {
        let manifold = self.mesh.materialize()?;
        let solver = self.solver.materialize_with_zones(&manifold, self.zones)?;
        let solver = match self.moving_wall {
            Some((axis, max_side, velocity)) => {
                solver.with_moving_wall(axis, max_side, velocity)?
            }
            None => solver,
        };
        let mut state = self.seed.apply(&solver, &manifold)?;

        let mut energy: Vec<R> = Vec::new();
        let mut divergence: Vec<R> = Vec::new();
        let mut max_speed: Vec<R> = Vec::new();

        collect(
            &self.observe,
            &manifold,
            &state,
            &mut energy,
            &mut divergence,
            &mut max_speed,
        )?;
        match self.stop {
            MarchStop::Fixed(n) => {
                for _ in 0..n {
                    let output = solver.step(&state)?;
                    state = output.into_state();
                    collect(
                        &self.observe,
                        &manifold,
                        &state,
                        &mut energy,
                        &mut divergence,
                        &mut max_speed,
                    )?;
                }
            }
            MarchStop::Steady { tol, max_steps } => {
                let mut prev_e = dec_kinetic_energy(&manifold, state.as_one_form())?;
                for _ in 0..max_steps {
                    let output = solver.step(&state)?;
                    state = output.into_state();
                    collect(
                        &self.observe,
                        &manifold,
                        &state,
                        &mut energy,
                        &mut divergence,
                        &mut max_speed,
                    )?;
                    let e = dec_kinetic_energy(&manifold, state.as_one_form())?;
                    if (e - prev_e).abs() < tol {
                        break;
                    }
                    prev_e = e;
                }
            }
        }

        let mut report = Report::new(self.name);
        if self.observe.kinetic_energy {
            report.add_series("kinetic_energy", energy);
        }
        if self.observe.divergence {
            report.add_series("divergence", divergence);
        }
        if self.observe.max_speed {
            report.add_series("max_speed", max_speed);
        }
        Ok(report)
    }
}

/// Sample the enabled diagnostics of `state` into their series. A free function
/// (not a closure) so it doesn't hold a borrow of the series across the march loop.
fn collect<const D: usize, R: CfdScalar>(
    observe: &Observe,
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    state: &SolenoidalField<R>,
    energy: &mut Vec<R>,
    divergence: &mut Vec<R>,
    max_speed: &mut Vec<R>,
) -> Result<(), PhysicsError> {
    let u = state.as_one_form();
    if observe.kinetic_energy {
        energy.push(dec_kinetic_energy(manifold, u)?);
    }
    if observe.divergence {
        divergence.push(dec_divergence_residual(manifold, u)?);
    }
    if observe.max_speed {
        max_speed.push(dec_max_speed(manifold, u)?);
    }
    Ok(())
}
