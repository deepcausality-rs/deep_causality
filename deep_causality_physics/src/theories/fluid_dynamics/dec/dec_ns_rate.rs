/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The DEC right-hand side of incompressible Navier–Stokes in rotational
//! (Lamb) form under Leray projection: `P(−i_u(du♭) − ν Δ_dR u♭ + g♭)`.
//!
//! The projector sits **inside** the rate (the governing equation of
//! `cfd-gap.md` §2), so the ODE the integrator marches is exactly the
//! projected dynamics on the divergence-free subspace — there is no
//! splitting error and no per-step energy discard. The unprojected
//! assembly is exposed separately for cross-validation and the pressure
//! diagnostic.

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, HodgeDecomposeOptions, LatticeComplex, LerayProjection, Manifold,
};

use crate::error::physics_error::PhysicsError;
use crate::quantities::fluid_dynamics::body_force_one_form::BodyForceOneForm;
use crate::quantities::fluid_dynamics::velocity_one_form::VelocityOneForm;
use crate::theories::fluid_dynamics::dec::DecNsScalar;

/// The rate field `u♭ ↦ −i_u(du♭) − ν Δ_dR u♭ + g♭` on a metric-bearing
/// periodic lattice manifold.
///
/// Construction validates every operator precondition — metric present,
/// lattice dimension at least 2 (the convective term needs grade-2 cells),
/// body-force edge count matching the lattice, `ν` finite and
/// non-negative — so that [`eval`](Self::eval) is **infallible**
/// (`Fn(&S) -> S`) and composes directly with
/// `deep_causality_calculus::Rk4`. Internal operator `Result`s are
/// unwrapped against these construction-time invariants; each unwrap
/// documents the invariant that makes it unreachable.
///
/// The viscous sign follows the Stage 0 pin: on a flat torus the
/// Hodge–de Rham Laplacian satisfies `Δ_dR = −∇²`, so the physical
/// diffusion `+ν∇²u` enters as `−ν Δ_dR u♭`.
///
/// Each evaluation assembles a scratch manifold carrying the state in its
/// grade-1 slice (the topology operators read the manifold's own data).
/// The lattice clone resets its lazy coboundary cache, so the operators
/// rebuild their sparse matrices per evaluation — a constant-factor cost
/// the projection CG dominates, accepted for the prototype and logged as
/// performance follow-up territory.
#[derive(Debug)]
pub struct DecNsRate<'m, const D: usize, R: RealField> {
    manifold: &'m Manifold<LatticeComplex<D, R>, R>,
    nu: R,
    body_force: Option<CausalTensor<R>>,
    /// Cell counts cached at construction: vertex count, edge count, and
    /// the total cell count of the full graded slab.
    n0: usize,
    n1: usize,
    total: usize,
}

impl<'m, const D: usize, R: DecNsScalar> DecNsRate<'m, D, R> {
    /// Builds the rate field, validating every per-step precondition once.
    ///
    /// # Errors
    /// * `PhysicsError::DimensionMismatch` when `D < 2` or the body-force
    ///   edge count does not match the lattice.
    /// * `PhysicsError::PhysicalInvariantBroken` when `ν` is negative.
    /// * `PhysicsError::NumericalInstability` when `ν` is not finite.
    /// * `PhysicsError::TopologyError` when the manifold carries no metric.
    pub fn new(
        manifold: &'m Manifold<LatticeComplex<D, R>, R>,
        nu: R,
        body_force: Option<&BodyForceOneForm<R>>,
    ) -> Result<Self, PhysicsError> {
        if D < 2 {
            return Err(PhysicsError::DimensionMismatch(format!(
                "DecNsRate requires a lattice of dimension >= 2 (the convective \
                 term contracts a grade-2 vorticity), got D = {D}"
            )));
        }
        if manifold.metric().is_none() {
            return Err(PhysicsError::TopologyError(
                "DecNsRate requires a metric-bearing manifold (Hodge star); \
                 construct it with CubicalReggeGeometry"
                    .into(),
            ));
        }
        if !nu.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "DecNsRate: viscosity must be finite".into(),
            ));
        }
        if nu < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DecNsRate: viscosity cannot be negative".into(),
            ));
        }

        let complex = manifold.complex();
        let n0 = complex.num_cells(0);
        let n1 = complex.num_cells(1);
        let total: usize = (0..=D).map(|g| complex.num_cells(g)).sum();

        let body_force = match body_force {
            Some(g) => {
                if g.len() != n1 {
                    return Err(PhysicsError::DimensionMismatch(format!(
                        "DecNsRate: body force carries {} edge coefficients, \
                         the lattice has {n1}",
                        g.len()
                    )));
                }
                Some(g.as_tensor().clone())
            }
            None => None,
        };

        Ok(Self {
            manifold,
            nu,
            body_force,
            n0,
            n1,
            total,
        })
    }

    /// The kinematic viscosity this rate was built with.
    pub fn nu(&self) -> R {
        self.nu
    }

    /// Evaluates `P(−i_u(du♭) − ν Δ_dR u♭ + g♭)`: the projected rate the
    /// integrator marches. One gauge-fixed CG solve per evaluation.
    ///
    /// # Errors
    /// `PhysicsError::TopologyError` when the projection CG does not
    /// converge within the supplied budget.
    pub fn eval_projected(
        &self,
        u: &VelocityOneForm<R>,
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<VelocityOneForm<R>, PhysicsError> {
        let raw = self.eval_unprojected(u);
        let projection = self.project_raw(&raw, opts)?;
        let (projected, _potential) = projection.into_parts();
        Ok(VelocityOneForm::from_raw(projected))
    }

    /// [`Self::eval_projected`], additionally returning the grade-0
    /// potential of the discarded gradient part — the Bernoulli-pressure
    /// input of the opt-in diagnostic (`dφ = −∇(p + ½|u|²)` at `ρ = 1`).
    pub(crate) fn eval_projected_with_potential(
        &self,
        u: &VelocityOneForm<R>,
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<(VelocityOneForm<R>, CausalTensor<R>), PhysicsError> {
        let raw = self.eval_unprojected(u);
        let projection = self.project_raw(&raw, opts)?;
        let (projected, potential) = projection.into_parts();
        Ok((VelocityOneForm::from_raw(projected), potential))
    }

    /// The shared projection of a raw RHS evaluation.
    fn project_raw(
        &self,
        raw: &VelocityOneForm<R>,
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<LerayProjection<R>, PhysicsError> {
        self.manifold
            .leray_project_opts(raw.as_tensor(), opts)
            .map_err(|e| PhysicsError::TopologyError(format!("Leray projection failed: {e}")))
    }

    /// Evaluates the **unprojected** assembly `−i_u(du♭) − ν Δ_dR u♭ + g♭`.
    ///
    /// Infallible by the construction-time validation; see the type doc.
    /// This is the cross-validation surface (the pointwise oracle has no
    /// projector) and the pressure diagnostic's input — not the marching
    /// rate.
    pub fn eval_unprojected(&self, u: &VelocityOneForm<R>) -> VelocityOneForm<R> {
        debug_assert_eq!(
            u.len(),
            self.n1,
            "marching state length is invariant under Add/Mul and validated at seeding"
        );

        // Scratch manifold carrying u in its grade-1 slice: the topology
        // operators (d, Δ) read the manifold's own data.
        let mut slab = vec![R::zero(); self.total];
        slab[self.n0..self.n0 + self.n1].copy_from_slice(u.as_tensor().as_slice());
        let data = CausalTensor::new(slab, vec![self.total])
            // Coverage exemption: a 1-D tensor of the validated slab length
            // cannot fail to allocate.
            .expect("1-D tensor allocation cannot fail");
        let metric = self
            .manifold
            .metric()
            // Coverage exemption: metric presence is validated at construction.
            .expect("metric presence validated at construction")
            .clone();
        let m_u =
            Manifold::from_cubical_with_metric(self.manifold.complex().clone(), data, metric, 0);

        // ω = d u♭ (grade-2), then the convective contraction i_u ω.
        let du = m_u.exterior_derivative(1);
        let conv = self
            .manifold
            .interior_product(u.as_tensor(), &du, 2)
            // Coverage exemption: grade (2 <= D) and operand lengths are
            // fixed by construction; interior_product cannot reject them.
            .expect("interior_product preconditions validated at construction");

        // Δ_dR u♭ (grade-1), with the pinned sign: −ν Δ_dR realizes +ν∇².
        let lap = m_u.laplacian(1);

        let conv_s = conv.as_slice();
        let lap_s = lap.as_slice();
        let rhs: Vec<R> = match &self.body_force {
            Some(g) => {
                let g_s = g.as_slice();
                (0..self.n1)
                    .map(|i| R::zero() - conv_s[i] - self.nu * lap_s[i] + g_s[i])
                    .collect()
            }
            None => (0..self.n1)
                .map(|i| R::zero() - conv_s[i] - self.nu * lap_s[i])
                .collect(),
        };

        let tensor = CausalTensor::new(rhs, vec![self.n1])
            // Coverage exemption: a 1-D tensor of the validated edge count
            // cannot fail to allocate.
            .expect("1-D tensor allocation cannot fail");
        VelocityOneForm::from_raw(tensor)
    }
}
