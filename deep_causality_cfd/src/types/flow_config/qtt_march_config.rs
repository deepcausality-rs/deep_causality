/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `QttMarchConfig` — the owned **configuration container** for a QTT 2-D incompressible marching
//! case (the "what"), separate from the [`CfdFlow`](crate::CfdFlow) workflow DSL (the "how").
//!
//! It is the QTT sibling of [`MarchConfig`](crate::MarchConfig): pure owned data sized to a
//! power-of-two periodic grid, holding the grid metadata, the solver parameters, the **owned seed
//! velocity fields** (materialized from a closure over the grid at build time — a velocity field is
//! data, not an analytic tag), the march-stop, and the observe set. Built by
//! [`QttMarchConfigBuilder`]; composed and run by [`CfdFlow::march`](crate::CfdFlow).

use crate::CfdScalar;
use crate::types::flow_config::MarchStop;
use alloc::format;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, Truncation};

/// The set of tensor-train-native diagnostics a QTT march collects into its `Report`. Built fluently;
/// each is a one-value-per-step series. No immersed-body / probe / centerline options — those need a
/// body the periodic QTT solver does not yet encode.
#[derive(Debug, Clone, Copy, Default)]
pub struct QttObserve {
    pub(crate) kinetic_energy: bool,
    pub(crate) divergence: bool,
    pub(crate) max_speed: bool,
    pub(crate) bond: bool,
    pub(crate) drag: bool,
    pub(crate) electron_density: bool,
    pub(crate) plasma_frequency: bool,
    pub(crate) heat_flux: bool,
    pub(crate) blackout_dwell: bool,
}

impl QttObserve {
    /// Collect the kinetic-energy series (`½(‖u‖² + ‖v‖²)`, one sample per step plus the seed).
    pub fn kinetic_energy(mut self) -> Self {
        self.kinetic_energy = true;
        self
    }

    /// Collect the divergence-residual series (`‖∇·u‖`, the post-projection incompressibility error).
    pub fn divergence(mut self) -> Self {
        self.divergence = true;
        self
    }

    /// Collect the maximum-speed series (`max √(u² + v²)`).
    pub fn max_speed(mut self) -> Self {
        self.max_speed = true;
        self
    }

    /// Collect the maximum-bond-dimension series (the compression / rank metric).
    pub fn bond(mut self) -> Self {
        self.bond = true;
        self
    }

    /// Collect the drag/lift coefficient series on the immersed body (requires the config to carry a
    /// body; the reference speed/length come from it). No-op without a body.
    pub fn drag(mut self) -> Self {
        self.drag = true;
        self
    }

    /// Collect the peak electron-density series `n_e` (the blackout coupling host, `run_coupled`).
    pub fn electron_density(mut self) -> Self {
        self.electron_density = true;
        self
    }

    /// Collect the (angular) plasma-frequency series `ω_p` (the blackout coupling host).
    pub fn plasma_frequency(mut self) -> Self {
        self.plasma_frequency = true;
        self
    }

    /// Collect the sensed heat-flux series — the first cell of the `"heat_flux"` scalar a loads
    /// stage publishes each coupled step (the blackout coupling host). No-op without a producer.
    pub fn heat_flux(mut self) -> Self {
        self.heat_flux = true;
        self
    }

    /// Collect the blackout dwell — the total time the GNSS/comms-denied flag is raised.
    pub fn blackout_dwell(mut self) -> Self {
        self.blackout_dwell = true;
        self
    }
}

/// An immersed body for the QTT march: the (smoothed) volume-fraction mask, the body velocity, the
/// Brinkman penalization `eta`, and the reference speed/length the drag coefficients use.
pub struct QttBody<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    pub(crate) mask: CausalTensorTrain<R>,
    pub(crate) ubx: R,
    pub(crate) uby: R,
    pub(crate) eta: R,
    /// The temperature the penalization holds the body at — the thermal analogue of `(ubx, uby)`,
    /// and the `T_w` the penalization heat integral is defined against. Defaults to zero (set it
    /// with [`QttMarchConfigBuilder::wall_temperature`]), which is what the observable used
    /// unconditionally before it was configurable.
    pub(crate) t_wall: R,
    pub(crate) u_ref: R,
    pub(crate) d_ref: R,
}

/// The owned configuration container for a QTT 2-D incompressible marching case. Holds only owned
/// specs; the same config can be run repeatedly (factual + counterfactual).
pub struct QttMarchConfig<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    pub(crate) name: String,
    pub(crate) lx: usize,
    pub(crate) ly: usize,
    pub(crate) dx: R,
    pub(crate) dy: R,
    pub(crate) dt: R,
    pub(crate) nu: R,
    pub(crate) trunc: Truncation<R>,
    pub(crate) u0: CausalTensor<R>,
    pub(crate) v0: CausalTensor<R>,
    pub(crate) stop: MarchStop<R>,
    pub(crate) observe: QttObserve,
    pub(crate) body: Option<QttBody<R>>,
}

impl<R> QttMarchConfig<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// The case name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The grid mode counts `(Lx, Ly)` (the grid is `2^Lx × 2^Ly`).
    pub fn modes(&self) -> (usize, usize) {
        (self.lx, self.ly)
    }

    /// The seed `u`-velocity field (`[2^Lx, 2^Ly]`).
    pub fn seed_u(&self) -> &CausalTensor<R> {
        &self.u0
    }

    /// The seed `v`-velocity field (`[2^Lx, 2^Ly]`).
    pub fn seed_v(&self) -> &CausalTensor<R> {
        &self.v0
    }
}

/// Fluent builder for a [`QttMarchConfig`]. Set the grid and solver, supply a seed (a closure over the
/// grid or pre-built fields), then `build`. The seed is **materialized at build-supply time** into
/// owned fields, so `build` validates the grid is `2^Lx × 2^Ly` and the seed matches it.
pub struct QttMarchConfigBuilder<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    name: Option<String>,
    grid: Option<(usize, usize, R, R)>,
    solver: Option<(R, R, Truncation<R>)>,
    seed: Option<(CausalTensor<R>, CausalTensor<R>)>,
    t_wall: Option<R>,
    stop: Option<MarchStop<R>>,
    observe: QttObserve,
    body: Option<QttBody<R>>,
}

impl<R> Default for QttMarchConfigBuilder<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    fn default() -> Self {
        Self {
            name: None,
            grid: None,
            solver: None,
            seed: None,
            stop: None,
            observe: QttObserve::default(),
            body: None,
            t_wall: None,
        }
    }
}

impl<R> QttMarchConfigBuilder<R>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    /// A fresh builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Name the case (defaults to `"qtt_march"`).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the `2^Lx × 2^Ly` periodic grid of spacings `dx`/`dy`.
    pub fn grid(mut self, lx: usize, ly: usize, dx: R, dy: R) -> Self {
        self.grid = Some((lx, ly, dx, dy));
        self
    }

    /// Set the solver parameters: time step `dt`, kinematic viscosity `nu`, per-step round policy.
    pub fn solver(mut self, dt: R, nu: R, trunc: Truncation<R>) -> Self {
        self.solver = Some((dt, nu, trunc));
        self
    }

    /// Set the march-stop (defaults to `Fixed(1)`).
    pub fn stop(mut self, stop: MarchStop<R>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Set an immersed body (Brinkman volume penalization): the `[0,1]` mask, the body velocity
    /// `(ubx, uby)` (zero for a static wall), the penalization `eta`, and the drag reference speed
    /// `u_ref` / length `d_ref`. The run then marches the penalized solver and (with `observe.drag`)
    /// emits the drag/lift series.
    #[allow(clippy::too_many_arguments)]
    pub fn body(
        mut self,
        mask: CausalTensorTrain<R>,
        ubx: R,
        uby: R,
        eta: R,
        u_ref: R,
        d_ref: R,
    ) -> Self {
        self.body = Some(QttBody {
            mask,
            ubx,
            uby,
            eta,
            t_wall: self.t_wall.unwrap_or_else(R::zero),
            u_ref,
            d_ref,
        });
        self
    }

    /// Set the body's wall temperature `T_w` — the temperature the penalization holds the body at,
    /// and the reference the penalization heat integral is defined against.
    ///
    /// Separate from [`body`](Self::body) rather than a seventh argument to it, so the existing
    /// call sites are unchanged and the thermal property is named at the point it is set. Order
    /// against `body` does not matter. Defaults to zero when never called.
    pub fn wall_temperature(mut self, t_wall: R) -> Self {
        self.t_wall = Some(t_wall);
        if let Some(body) = self.body.as_mut() {
            body.t_wall = t_wall;
        }
        self
    }

    /// Set the observe set.
    pub fn observe(mut self, observe: QttObserve) -> Self {
        self.observe = observe;
        self
    }

    /// Materialize the seed velocity from a closure `f(x, y) -> (u, v)` evaluated over the grid (the
    /// grid must already be set). The fields are stored owned; `build` re-validates their shape.
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] if the grid is not yet set.
    pub fn seed_fn<F: Fn(R, R) -> (R, R)>(mut self, f: F) -> Result<Self, PhysicsError> {
        let (lx, ly, dx, dy) = self.grid.ok_or_else(|| {
            PhysicsError::DimensionMismatch("seed_fn requires the grid to be set first".into())
        })?;
        let (nx, ny) = (1usize << lx, 1usize << ly);
        let mut ud = alloc::vec![R::zero(); nx * ny];
        let mut vd = alloc::vec![R::zero(); nx * ny];
        for i in 0..nx {
            let x = R::from_usize(i).expect("a lattice index lifts into every real field") * dx;
            for j in 0..ny {
                let y = R::from_usize(j).expect("a lattice index lifts into every real field") * dy;
                let (u, v) = f(x, y);
                ud[i * ny + j] = u;
                vd[i * ny + j] = v;
            }
        }
        let shape = alloc::vec![nx, ny];
        self.seed = Some((
            CausalTensor::new(ud, shape.clone())?,
            CausalTensor::new(vd, shape)?,
        ));
        Ok(self)
    }

    /// Seed with the analytic 2-D Taylor–Green vortex `u = −cos(x)sin(y)`, `v = sin(x)cos(y)`
    /// (the grid must already be set) — a convenience over [`seed_fn`](Self::seed_fn).
    ///
    /// # Errors
    /// As [`seed_fn`](Self::seed_fn).
    pub fn taylor_green(self) -> Result<Self, PhysicsError> {
        self.seed_fn(|x, y| (R::zero() - x.cos() * y.sin(), x.sin() * y.cos()))
    }

    /// Supply pre-built seed velocity fields directly (`build` validates their shape).
    pub fn seed_fields(mut self, u0: CausalTensor<R>, v0: CausalTensor<R>) -> Self {
        self.seed = Some((u0, v0));
        self
    }

    /// Build the owned config.
    ///
    /// # Errors
    /// [`PhysicsError::DimensionMismatch`] if the grid, solver, or seed is missing, or if a seed
    /// field's shape does not match the `2^Lx × 2^Ly` grid.
    pub fn build(self) -> Result<QttMarchConfig<R>, PhysicsError> {
        let (lx, ly, dx, dy) = self
            .grid
            .ok_or_else(|| PhysicsError::DimensionMismatch("qtt_march: grid not set".into()))?;
        let (dt, nu, trunc) = self
            .solver
            .ok_or_else(|| PhysicsError::DimensionMismatch("qtt_march: solver not set".into()))?;
        let (u0, v0) = self
            .seed
            .ok_or_else(|| PhysicsError::DimensionMismatch("qtt_march: seed not set".into()))?;

        let want = [1usize << lx, 1usize << ly];
        for f in [&u0, &v0] {
            if f.shape() != want {
                return Err(PhysicsError::DimensionMismatch(format!(
                    "qtt_march: seed shape {:?} does not match the grid {want:?}",
                    f.shape()
                )));
            }
        }

        Ok(QttMarchConfig {
            name: self.name.unwrap_or_else(|| "qtt_march".into()),
            lx,
            ly,
            dx,
            dy,
            dt,
            nu,
            trunc,
            u0,
            v0,
            stop: self.stop.unwrap_or(MarchStop::Fixed(1)),
            observe: self.observe,
            body: self.body,
        })
    }
}
