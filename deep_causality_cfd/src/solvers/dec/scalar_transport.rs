/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Passive scalar advection–diffusion on the DEC manifold — the temperature field a genuine
//! Fourier-law wall heat flux differentiates.
//!
//! The scalar `T` is a **0-cochain** (one value per vertex) marched under
//!
//! ```text
//! ∂T/∂t = −i_u(dT) − κ·Δ_dR T,
//! ```
//!
//! which is the DEC statement of `∂T/∂t + u·∇T = κ∇²T`. Both terms reuse the operators the momentum
//! rate already uses, at a different grade:
//!
//! * **Advection.** For a 0-form `T`, `dT` is a 1-form and `i_u(dT)` is the 0-form `u·∇T` — the same
//!   interior product `DecNsRate` contracts against the vorticity 2-form for the Lamb vector.
//! * **Diffusion.** `Δ_dR` on 0-forms is `δd`. The sign follows the crate's Stage-0 pin: on a flat
//!   torus `Δ_dR = −∇²`, so physical diffusion `+κ∇²T` enters as `−κ·Δ_dR T`, exactly as `+ν∇²u`
//!   enters the momentum rate as `−ν·Δ_dR u♭`.
//!
//! Sharing the operators is deliberate. A scalar and a velocity component are then differentiated by
//! the same code, so a sign or grade error moves both and the momentum path's existing verification
//! partly covers the scalar too. The audit's §4b records the opposite arrangement as a defect — a gate
//! that "tests a re-implementation, not the shipped solver" — and a parallel discretisation here is
//! how that arises.
//!
//! The scalar is **passive**: it reads the velocity and does not feed back. There is no buoyancy term,
//! so the momentum path is untouched.

use alloc::format;
use alloc::vec::Vec;

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    CellClass, ChainComplex, CutCellRegistry, LatticeCell, LatticeComplex, Manifold,
};

use crate::solvers::dec::DecNsScalar;
use deep_causality_physics::PhysicsError;

/// Passive scalar advection–diffusion on a metric-bearing lattice manifold.
///
/// Construction validates the per-step preconditions once — metric present, `κ` finite and
/// non-negative — matching the DEC family's envelope-validation convention (the audit's §7 records
/// the QTT family's failure to do the same as a systemic gap).
#[derive(Debug)]
pub struct DecScalarRate<'m, const D: usize, R: DecNsScalar> {
    manifold: &'m Manifold<LatticeComplex<D, R>, R>,
    kappa: R,
    n0: usize,
    /// Vertices held at the wall temperature (the immersed body). Empty without a body.
    pinned: Vec<usize>,
    t_wall: R,
}

impl<'m, const D: usize, R: DecNsScalar> DecScalarRate<'m, D, R> {
    /// Builds the scalar rate for diffusivity `kappa`.
    ///
    /// # Errors
    /// * [`PhysicsError::TopologyError`] when the manifold carries no metric (the interior product
    ///   and the Laplacian are both metric-dependent).
    /// * [`PhysicsError::NumericalInstability`] when `kappa` is not finite.
    /// * [`PhysicsError::PhysicalInvariantBroken`] when `kappa` is negative — a negative diffusivity
    ///   is an anti-diffusion that amplifies every mode.
    pub fn new(
        manifold: &'m Manifold<LatticeComplex<D, R>, R>,
        kappa: R,
    ) -> Result<Self, PhysicsError> {
        if manifold.metric().is_none() {
            return Err(PhysicsError::TopologyError(
                "DecScalarRate requires a metric-bearing manifold (Hodge star); construct it with \
                 CubicalReggeGeometry"
                    .into(),
            ));
        }
        if !kappa.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "DecScalarRate: diffusivity must be finite".into(),
            ));
        }
        if kappa < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "DecScalarRate: diffusivity must be non-negative, got {kappa}"
            )));
        }
        Ok(Self {
            manifold,
            kappa,
            n0: manifold.complex().num_cells(0),
            pinned: Vec::new(),
            t_wall: R::zero(),
        })
    }

    /// Attaches an **isothermal immersed body**: every vertex of a non-`Fluid` cell is held at
    /// `t_wall`.
    ///
    /// The pinned set is derived from the same [`CutCellRegistry`] that supplies the momentum
    /// no-slip constraint, so the thermal and mechanical boundaries describe one body. Deriving both
    /// from one geometry makes that structural rather than a convention two call sites must keep.
    ///
    /// `Cut` cells are pinned along with `Solid` ones, so the body is isothermal including its
    /// surface layer. That is what makes the wall value the flux diagnostic anchors at a fragment
    /// centroid agree with the field the diagnostic samples one step away from it.
    ///
    /// # Errors
    /// [`PhysicsError::NumericalInstability`] when `t_wall` is not finite.
    pub fn with_isothermal_body(
        mut self,
        registry: &CutCellRegistry<D, R>,
        t_wall: R,
    ) -> Result<Self, PhysicsError> {
        if !t_wall.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "DecScalarRate: wall temperature must be finite".into(),
            ));
        }
        let complex = self.manifold.complex();
        let shape = complex.shape();
        let periodic = complex.periodic();

        // Vertex position → 0-cell index.
        let mut vertex_index = alloc::collections::BTreeMap::<[usize; D], usize>::new();
        for (idx, vertex) in complex.iter_cells(0).enumerate() {
            vertex_index.insert(*vertex.position(), idx);
        }
        let cells: Vec<LatticeCell<D>> = complex.iter_cells(D).collect();

        let mut pinned: Vec<usize> = Vec::new();
        for (&cell_id, cut) in registry.iter() {
            if cut.class() == CellClass::Fluid {
                continue;
            }
            let Some(cell) = cells.get(cell_id) else {
                continue;
            };
            let base = *cell.position();
            // The 2^D corners of the top cube, wrapped on periodic axes.
            for corner in 0..(1usize << D) {
                let mut pos = base;
                let mut inside = true;
                for (axis, p) in pos.iter_mut().enumerate() {
                    let bit = (corner >> axis) & 1;
                    *p += bit;
                    if *p >= shape[axis] {
                        if periodic[axis] {
                            *p %= shape[axis];
                        } else {
                            inside = false;
                        }
                    }
                }
                if inside && let Some(&v) = vertex_index.get(&pos) {
                    pinned.push(v);
                }
            }
        }
        pinned.sort_unstable();
        pinned.dedup();

        self.pinned = pinned;
        self.t_wall = t_wall;
        Ok(self)
    }

    /// The wall temperature the body is held at; zero without a body.
    pub fn wall_temperature(&self) -> R {
        self.t_wall
    }

    /// The vertices held at the wall temperature.
    pub fn pinned_vertices(&self) -> &[usize] {
        &self.pinned
    }

    /// Evaluates `∂T/∂t = −i_u(dT) − κ·Δ_dR T`, with the rate zeroed on pinned vertices so a
    /// constrained value cannot drift.
    ///
    /// `scalar` is the 0-cochain `T` (one value per vertex); `velocity` is the edge 1-cochain `u♭`
    /// the momentum solver marches.
    ///
    /// # Errors
    /// * [`PhysicsError::DimensionMismatch`] when either field has the wrong length.
    /// * [`PhysicsError::TopologyError`] wrapping an operator failure.
    pub fn eval(
        &self,
        scalar: &CausalTensor<R>,
        velocity: &CausalTensor<R>,
    ) -> Result<CausalTensor<R>, PhysicsError> {
        if scalar.len() != self.n0 {
            return Err(PhysicsError::DimensionMismatch(format!(
                "DecScalarRate: expected {} scalar values (one per vertex), got {}",
                self.n0,
                scalar.len()
            )));
        }
        let n1 = self.manifold.complex().num_cells(1);
        if velocity.len() != n1 {
            return Err(PhysicsError::DimensionMismatch(format!(
                "DecScalarRate: expected {} velocity edge values, got {}",
                n1,
                velocity.len()
            )));
        }

        // Advection: dT is a 1-form; i_u(dT) is the 0-form u·∇T.
        let grad = self.manifold.exterior_derivative_of(scalar.as_slice(), 0);
        let advect = self
            .manifold
            .interior_product(velocity, &grad, 1)
            .map_err(|e| PhysicsError::TopologyError(format!("interior_product(u, dT): {e}")))?;

        // Diffusion: +κ∇²T = −κ·Δ_dR T (Stage-0 sign pin, as the viscous term).
        let lap = self.manifold.laplacian_of(scalar.as_slice(), 0);

        let mut out = alloc::vec![R::zero(); self.n0];
        for (i, o) in out.iter_mut().enumerate() {
            *o = R::zero() - advect.as_slice()[i] - self.kappa * lap.as_slice()[i];
        }
        // A pinned vertex is Dirichlet: its value is prescribed, so its rate is zero.
        for &v in &self.pinned {
            if let Some(slot) = out.get_mut(v) {
                *slot = R::zero();
            }
        }

        CausalTensor::new(out, alloc::vec![self.n0]).map_err(|e| {
            PhysicsError::DimensionMismatch(format!("DecScalarRate: rate assembly: {e:?}"))
        })
    }

    /// Applies the Dirichlet wall condition in place, setting every pinned vertex to the wall
    /// temperature. A no-op without a body.
    pub fn apply_wall(&self, values: &mut [R]) {
        for &v in &self.pinned {
            if let Some(slot) = values.get_mut(v) {
                *slot = self.t_wall;
            }
        }
    }

    /// One explicit Euler step of the scalar, with the wall condition re-applied afterwards.
    ///
    /// Explicit Euler is deliberate for a **passive diagnostic** scalar: it keeps this path
    /// independent of the momentum integrator, so nothing here can perturb the velocity march. The
    /// stability limit is the usual `dt ≤ dx²/(2Dκ)`; callers marching to steady state for a wall-flux
    /// measurement are well inside it.
    ///
    /// # Errors
    /// As [`eval`](Self::eval), plus [`PhysicsError::PhysicalInvariantBroken`] when `dt` is not
    /// finite and positive.
    pub fn step(
        &self,
        scalar: &CausalTensor<R>,
        velocity: &CausalTensor<R>,
        dt: R,
    ) -> Result<CausalTensor<R>, PhysicsError> {
        if !dt.is_finite() || dt <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "DecScalarRate: dt must be finite and positive, got {dt}"
            )));
        }
        let rate = self.eval(scalar, velocity)?;
        let mut next = scalar.as_slice().to_vec();
        for (n, r) in next.iter_mut().zip(rate.as_slice()) {
            *n += dt * *r;
        }
        self.apply_wall(&mut next);
        CausalTensor::new(next, alloc::vec![self.n0]).map_err(|e| {
            PhysicsError::DimensionMismatch(format!("DecScalarRate: step assembly: {e:?}"))
        })
    }
}
