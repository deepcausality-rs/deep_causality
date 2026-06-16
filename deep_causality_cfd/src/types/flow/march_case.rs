/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::solvers::DecNsConfig;
use crate::solvers::dec::diagnostics::{
    dec_divergence_residual, dec_kinetic_energy, dec_max_speed, dec_sample_velocity,
};
use crate::solvers::dec::surface_force::{
    force_coefficient, pressure_surface_force, viscous_surface_force,
};
use crate::solvers::dec::{BoundaryZone, DecNsSolver};
use crate::traits::{Marcher, Solver};
use crate::types::flow::{CoupledField, Mesh, Observe, PhysicsStage, Report, Seed, StepContext};
use crate::types::{Ambient, CfdScalar};
use deep_causality_physics::{PhysicsError, SolenoidalField};
use deep_causality_topology::{
    ChainComplex, CutCellRegistry, LatticeCell, LatticeComplex, Manifold,
};

use alloc::collections::BTreeMap;

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
pub struct MarchCase<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>, C: PhysicsStage<D, R>> {
    name: String,
    mesh: Mesh<D, R>,
    solver: DecNsConfig<R>,
    /// A prescribed moving wall as `(axis, max_side, velocity)`, applied via the
    /// solver's `with_moving_wall` builder. Reuses the existing solver mechanism, so
    /// no type collides with the `MovingWall` boundary zone.
    moving_wall: Option<(usize, bool, [R; D])>,
    seed: Seed,
    stop: MarchStop<R>,
    observe: Observe<D, R>,
    zones: Z,
    /// The between-step multi-physics coupling (`()` for single-physics).
    coupling: C,
    /// Uniform initial coupled scalar fields, sized to the cell count at materialization.
    coupled_scalars: Vec<(String, R)>,
}

impl<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>, C: PhysicsStage<D, R>>
    MarchCase<D, R, Z, C>
{
    /// Assemble a marching case from its owned specs (constructed by `Flow::march`).
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
}

impl<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>, C: PhysicsStage<D, R>> Solver<R>
    for MarchCase<D, R, Z, C>
{
    /// Materialize the domain (with its boundary zones), seed, march, and collect the
    /// observed series. Borrows (manifold, solver, registry) stay inside this call.
    fn run(self) -> Result<Report<R>, PhysicsError> {
        let (manifold, registry) = self.mesh.materialize()?;
        let ref_len = self.mesh.frontal_length();
        if self.observe.drag.is_some() && registry.is_none() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "Flow::march: drag/lift observed but the mesh carries no immersed body".into(),
            ));
        }
        let solver = self.solver.materialize_with_zones(&manifold, self.zones)?;
        let solver = match self.moving_wall {
            Some((axis, max_side, velocity)) => {
                solver.with_moving_wall(axis, max_side, velocity)?
            }
            None => solver,
        };
        let mut state = self.seed.apply(&solver, &manifold)?;

        // The between-step coupling carrier: the per-step ambient (ν the marcher reads) plus any
        // seeded coupled scalar fields. With the `()` coupling the ambient stays at the
        // construction ν, so the march reproduces the single-physics path bit-for-bit.
        let dt = solver.dt();
        let ncells = manifold.complex().num_cells(D);
        let mut field = CoupledField::new(Ambient::new(solver.nu(), R::zero(), None));
        for (name, value) in &self.coupled_scalars {
            field.set_scalar(name.clone(), vec![*value; ncells]);
        }

        let mut series = Series::new();
        let ctx = Context {
            observe: &self.observe,
            manifold: &manifold,
            registry: registry.as_ref(),
            solver: &solver,
            ref_len,
        };

        ctx.sample(&state, &mut series)?;
        match self.stop {
            MarchStop::Fixed(n) => {
                for s in 0..n {
                    state = advance_coupled(
                        &solver,
                        &manifold,
                        &self.coupling,
                        &mut field,
                        &state,
                        dt,
                        s + 1,
                    )?;
                    ctx.sample(&state, &mut series)?;
                }
            }
            MarchStop::Steady { tol, max_steps } => {
                let mut prev_e = dec_kinetic_energy(&manifold, state.as_one_form())?;
                for s in 0..max_steps {
                    state = advance_coupled(
                        &solver,
                        &manifold,
                        &self.coupling,
                        &mut field,
                        &state,
                        dt,
                        s + 1,
                    )?;
                    ctx.sample(&state, &mut series)?;
                    let e = dec_kinetic_energy(&manifold, state.as_one_form())?;
                    if (e - prev_e).abs() < tol {
                        break;
                    }
                    prev_e = e;
                }
            }
        }

        let mut report = Report::new(self.name);
        series.into_report(&self.observe, &mut report);
        // The centerline is a one-shot final-state profile, not a per-step series.
        if let Some(axis) = self.observe.centerline {
            report.add_series("centerline", centerline_profile(&manifold, &state, axis)?);
        }
        Ok(report)
    }
}

/// Run the between-step coupling, then advance one projected step under the resulting ambient.
/// The coupling reads the current state through a [`StepContext`] and mutates the [`CoupledField`]
/// (scalars + the ambient `ν` the marcher then reads). An error in any stage short-circuits the
/// step.
fn advance_coupled<const D: usize, R: CfdScalar, C: PhysicsStage<D, R>>(
    solver: &DecNsSolver<'_, D, R>,
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    coupling: &C,
    field: &mut CoupledField<R>,
    state: &SolenoidalField<R>,
    dt: R,
    step: usize,
) -> Result<SolenoidalField<R>, PhysicsError> {
    let ctx = StepContext::new(manifold, state, dt, step);
    coupling.apply(&ctx, field)?;
    Ok(solver.advance(state, field.ambient())?.into_state())
}

/// The per-step observation context — the immutable run state the sampler reads.
struct Context<'a, const D: usize, R: CfdScalar> {
    observe: &'a Observe<D, R>,
    manifold: &'a Manifold<LatticeComplex<D, R>, R>,
    registry: Option<&'a CutCellRegistry<D, R>>,
    solver: &'a DecNsSolver<'a, D, R>,
    ref_len: Option<R>,
}

impl<'a, const D: usize, R: CfdScalar> Context<'a, D, R> {
    /// Sample every enabled diagnostic of `state` into the series accumulator.
    fn sample(
        &self,
        state: &SolenoidalField<R>,
        series: &mut Series<R>,
    ) -> Result<(), PhysicsError> {
        let u = state.as_one_form();
        if self.observe.kinetic_energy {
            series.energy.push(dec_kinetic_energy(self.manifold, u)?);
        }
        if self.observe.divergence {
            series
                .divergence
                .push(dec_divergence_residual(self.manifold, u)?);
        }
        if self.observe.max_speed {
            series.max_speed.push(dec_max_speed(self.manifold, u)?);
        }
        if let (Some(u_ref), Some(registry), Some(ref_len)) =
            (self.observe.drag, self.registry, self.ref_len)
        {
            let (cd, cl) = surface_force_coefficients(
                self.manifold,
                registry,
                self.solver,
                state,
                u_ref,
                ref_len,
            )?;
            series.drag.push(cd);
            series.lift.push(cl);
        }
        if let Some(point) = self.observe.probe {
            let v = dec_sample_velocity(self.manifold, u, &point)?;
            // The transverse component (axis 1) is the wake-oscillation signal.
            series.probe.push(if D > 1 { v[1] } else { v[0] });
        }
        Ok(())
    }
}

/// The per-step series an observation accumulates. Allocated only for enabled diagnostics
/// at `into_report`; the vectors stay empty for the disabled ones.
struct Series<R: CfdScalar> {
    energy: Vec<R>,
    divergence: Vec<R>,
    max_speed: Vec<R>,
    drag: Vec<R>,
    lift: Vec<R>,
    probe: Vec<R>,
}

impl<R: CfdScalar> Series<R> {
    fn new() -> Self {
        Self {
            energy: Vec::new(),
            divergence: Vec::new(),
            max_speed: Vec::new(),
            drag: Vec::new(),
            lift: Vec::new(),
            probe: Vec::new(),
        }
    }

    fn into_report<const D: usize>(self, observe: &Observe<D, R>, report: &mut Report<R>) {
        if observe.kinetic_energy {
            report.add_series("kinetic_energy", self.energy);
        }
        if observe.divergence {
            report.add_series("divergence", self.divergence);
        }
        if observe.max_speed {
            report.add_series("max_speed", self.max_speed);
        }
        if observe.drag.is_some() {
            report.add_series("drag", self.drag);
            report.add_series("lift", self.lift);
        }
        if observe.probe.is_some() {
            report.add_series("probe", self.probe);
        }
    }
}

/// The (drag, lift) coefficients on the immersed body at the given state: the total
/// hydrodynamic force `F = F_pressure + F_viscous`, nondimensionalized by
/// `½ρU²L` (`ρ = 1`, `U = u_ref`, `L = ref_len`). The pressure term reads the static
/// pressure at the cut cells (the per-cell average of its corner-vertex 0-form,
/// recovered by the solver's one-CG-solve pressure diagnostic); the viscous term reads
/// the wall shear off the velocity field. Drag is the streamwise (axis-0) component,
/// lift the transverse (axis-1) component.
fn surface_force_coefficients<const D: usize, R: CfdScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    registry: &CutCellRegistry<D, R>,
    solver: &DecNsSolver<'_, D, R>,
    state: &SolenoidalField<R>,
    u_ref: R,
    ref_len: R,
) -> Result<(R, R), PhysicsError> {
    let (_bernoulli, static_p) = solver.pressure_diagnostic(state)?;

    // Vertex pressure keyed by lattice position, and the top cells in registry-key order.
    let pressure: BTreeMap<[usize; D], R> = manifold
        .complex()
        .iter_cells(0)
        .zip(static_p.as_tensor().as_slice().iter())
        .map(|(vertex, &p)| (*vertex.position(), p))
        .collect();
    let cells: Vec<LatticeCell<D>> = manifold.complex().iter_cells(D).collect();
    let inv_corners = R::one()
        / R::from_usize(1usize << D).expect("the corner count lifts into every real field");

    // Cell-centered static pressure = the mean of the cell's 2^D corner vertices.
    let cell_pressure = |cell_id: usize| -> R {
        let base = *cells[cell_id].position();
        let mut sum = R::zero();
        for corner in 0..(1usize << D) {
            let mut pos = base;
            for (j, p) in pos.iter_mut().enumerate() {
                *p += (corner >> j) & 1;
            }
            if let Some(&pv) = pressure.get(&pos) {
                sum += pv;
            }
        }
        sum * inv_corners
    };

    let fp = pressure_surface_force(registry, cell_pressure);
    let fv = viscous_surface_force(manifold, registry, state.as_one_form(), solver.nu())?;
    let drag = force_coefficient(fp[0] + fv[0], u_ref, ref_len);
    let lift = if D > 1 {
        force_coefficient(fp[1] + fv[1], u_ref, ref_len)
    } else {
        R::zero()
    };
    Ok((drag, lift))
}

/// The final-state velocity profile along the domain-centered line parallel to `axis`:
/// the streamwise-neighbour component `u[(axis + 1) mod D]` sampled at every lattice
/// node along `axis`, with the other coordinates pinned to the domain center. This is
/// the Ghia lid-cavity centerline comparison (vertical centerline → `u_x(y)`).
fn centerline_profile<const D: usize, R: CfdScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    state: &SolenoidalField<R>,
    axis: usize,
) -> Result<Vec<R>, PhysicsError> {
    if axis >= D {
        return Err(PhysicsError::DimensionMismatch(format!(
            "centerline: axis {axis} out of range for D = {D}"
        )));
    }
    let shape = manifold.complex().shape();
    let dx = manifold
        .metric()
        .and_then(|g| g.axis_lengths())
        .ok_or_else(|| {
            PhysicsError::TopologyError(
                "centerline requires an axis-aligned geometry (per-axis spacing)".into(),
            )
        })?;
    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    let component = (axis + 1) % D;
    let u = state.as_one_form();
    let mut profile = Vec::with_capacity(shape[axis]);
    for i in 0..shape[axis] {
        let mut point = [R::zero(); D];
        for (a, p) in point.iter_mut().enumerate() {
            *p = if a == axis {
                R::from_usize(i).expect("a lattice index lifts into every real field") * dx[a]
            } else {
                // Domain center along the off-axis directions.
                R::from_usize(shape[a]).expect("a lattice extent lifts into R") * half * dx[a]
            };
        }
        let v = dec_sample_velocity(manifold, u, &point)?;
        profile.push(v[component]);
    }
    Ok(profile)
}
