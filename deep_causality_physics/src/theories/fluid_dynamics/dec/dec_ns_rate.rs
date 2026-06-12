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

use core::cell::RefCell;
use deep_causality_tensor::CausalTensor;

use deep_causality_topology::{
    ChainComplex, DecStencilTables, HodgeDecomposeOptions, LatticeComplex, LerayProjection,
    Manifold,
};

use crate::error::physics_error::PhysicsError;
use crate::quantities::fluid_dynamics::body_force_one_form::BodyForceOneForm;
use crate::quantities::fluid_dynamics::velocity_one_form::VelocityOneForm;
use crate::theories::fluid_dynamics::dec::DecNsScalar;
use crate::theories::fluid_dynamics::dec::spectral_diffusion::SpectralDiffusion;

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
/// Each evaluation applies the operators directly on the marching field
/// through the topology crate's `_of` variants — no scratch manifold and
/// no data-slab copy per stage; the memoized sparse matrices are shared
/// through the borrowed manifold.
#[derive(Debug)]
pub struct DecNsRate<'m, const D: usize, R: DecNsScalar> {
    manifold: &'m Manifold<LatticeComplex<D, R>, R>,
    nu: R,
    body_force: Option<CausalTensor<R>>,
    /// Edge count cached at construction (the marching state's length).
    n1: usize,
    /// Compiled stencil engine (tables + reusable workspace). `Some` by
    /// default — the fused streaming path, equivalence-gated against the
    /// generic composition; `None` evaluates through the generic
    /// compositional operators (the oracle path, kept for
    /// cross-validation and benchmarking via
    /// [`Self::with_generic_assembly`]).
    engine: Option<StencilEngine<R>>,
    /// Opt-in spectral viscous evaluation (fully periodic lattices only;
    /// the `spectral-diffusion` capability). `None` by default.
    spectral: Option<SpectralDiffusion<R>>,
}

/// The compiled tables plus the per-evaluation scratch. `RefCell`: the
/// rate is evaluated from a single orchestration thread (`Rk4` stages are
/// sequential); the operator kernels parallelize *internally* under the
/// `parallel` feature while the workspace borrow is exclusive.
#[derive(Debug)]
struct StencilEngine<R> {
    tables: DecStencilTables<R>,
    ws: RefCell<RateWorkspace<R>>,
}

#[derive(Debug)]
struct RateWorkspace<R> {
    omega: Vec<R>,
    pre: Vec<R>,
    wedge: Vec<R>,
    conv: Vec<R>,
    visc_a: Vec<R>,
    visc_b: Vec<R>,
    s0: Vec<R>,
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
        let n1 = complex.num_cells(1);

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

        let tables = DecStencilTables::compile(manifold)
            .map_err(|e| PhysicsError::TopologyError(format!("stencil compilation failed: {e}")))?;
        let n0 = complex.num_cells(0);
        let n2 = complex.num_cells(2);
        let (pre_len, wedge_len) = tables.convective_scratch_lens();
        let ws = RateWorkspace {
            omega: vec![R::zero(); n2],
            pre: vec![R::zero(); pre_len],
            wedge: vec![R::zero(); wedge_len],
            conv: vec![R::zero(); n1],
            visc_a: vec![R::zero(); n1],
            visc_b: vec![R::zero(); n1],
            s0: vec![R::zero(); n0],
        };
        let engine = Some(StencilEngine {
            tables,
            ws: RefCell::new(ws),
        });

        Ok(Self {
            manifold,
            nu,
            body_force,
            n1,
            engine,
            spectral: None,
        })
    }

    /// Opt into the spectral evaluation of the viscous term (fully
    /// periodic uniform lattices only). Off by default; the validation
    /// ladder gates any future default-on.
    ///
    /// # Errors
    /// `PhysicsError::TopologyError` when the lattice is not fully
    /// periodic or the metric carries no per-axis Euclidean spacings.
    pub fn with_spectral_diffusion(mut self) -> Result<Self, PhysicsError> {
        self.spectral = Some(SpectralDiffusion::new(self.manifold)?);
        Ok(self)
    }

    /// Switch this rate to the generic compositional operator path — the
    /// equivalence oracle and the benchmark baseline. The default is the
    /// compiled stencil pipeline.
    pub fn with_generic_assembly(mut self) -> Self {
        self.engine = None;
        self
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

        if let Some(engine) = &self.engine {
            return self.eval_unprojected_fused(engine, u);
        }

        // The operators evaluate directly on the marching field through
        // the `_of` variants — no scratch manifold, no data-slab copy.
        let u_slice = u.as_tensor().as_slice();

        // ω = d u♭ (grade-2), then the convective contraction i_u ω.
        let du = self.manifold.exterior_derivative_of(u_slice, 1);
        let conv = self
            .manifold
            .interior_product(u.as_tensor(), &du, 2)
            // Coverage exemption: grade (2 <= D) and operand lengths are
            // fixed by construction; interior_product cannot reject them.
            .expect("interior_product preconditions validated at construction");

        // Δ_dR u♭ (grade-1), with the pinned sign: −ν Δ_dR realizes +ν∇².
        let lap = self.manifold.laplacian_of(u_slice, 1);

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

    /// The fused streaming assembly over the compiled stencil tables: six
    /// gather passes through the reusable workspace, no intermediate
    /// tensor, one output allocation. Equivalence to the generic path is
    /// pinned by `stencil_tests.rs` (topology) and the rate tests here.
    fn eval_unprojected_fused(
        &self,
        engine: &StencilEngine<R>,
        u: &VelocityOneForm<R>,
    ) -> VelocityOneForm<R> {
        let u_slice = u.as_tensor().as_slice();
        let t = &engine.tables;
        let mut ws = engine.ws.borrow_mut();
        let ws = &mut *ws;

        // Coverage exemptions on the unwraps below: every buffer length is
        // fixed at construction from the same tables, so the length
        // validation cannot fail.
        t.apply_d1(u_slice, &mut ws.omega)
            .expect("workspace lengths fixed at construction");
        t.apply_convective(&ws.omega, u_slice, &mut ws.pre, &mut ws.wedge, &mut ws.conv)
            .expect("workspace lengths fixed at construction");
        if let Some(spectral) = &self.spectral {
            // Δ₁u in one spectral pass; visc_b is unused on this path.
            spectral.apply_laplacian_1(u_slice, &mut ws.visc_a);
            for v in ws.visc_b.iter_mut() {
                *v = R::zero();
            }
        } else {
            t.apply_delta2(&ws.omega, &mut ws.visc_a)
                .expect("workspace lengths fixed at construction");
            t.apply_delta1(u_slice, &mut ws.s0)
                .expect("workspace lengths fixed at construction");
            t.apply_d0(&ws.s0, &mut ws.visc_b)
                .expect("workspace lengths fixed at construction");
        }

        let rhs: Vec<R> = match &self.body_force {
            Some(g) => {
                let g_s = g.as_slice();
                (0..self.n1)
                    .map(|i| {
                        R::zero() - ws.conv[i] - self.nu * (ws.visc_a[i] + ws.visc_b[i]) + g_s[i]
                    })
                    .collect()
            }
            None => (0..self.n1)
                .map(|i| R::zero() - ws.conv[i] - self.nu * (ws.visc_a[i] + ws.visc_b[i]))
                .collect(),
        };

        let tensor = CausalTensor::new(rhs, vec![self.n1])
            // Coverage exemption: a 1-D tensor of the validated edge count
            // cannot fail to allocate.
            .expect("1-D tensor allocation cannot fail");
        VelocityOneForm::from_raw(tensor)
    }
}
