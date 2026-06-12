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
use deep_causality_num::FromPrimitive;
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
/// non-negative — so that [`eval_projected`](Self::eval_projected) is **infallible**
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
    /// Wall-tangential edges the no-slip condition pins to zero (the
    /// no-slip-viscous capability). Empty on fully periodic lattices, where
    /// every projection runs the unconstrained (spectral-dispatch) path
    /// bit-unchanged; on wall-bounded lattices `project_raw` routes through
    /// the constrained Leray projector instead.
    no_slip: super::dec_ns_solver::no_slip::NoSlipConstraint,
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
    /// Adjoint-chain scratch of the skew-symmetrized convective term
    /// (dec-ns-stability): `G*_ω u` and the two transposed-stage buffers.
    adj_corr: Vec<R>,
    adj_s1: Vec<R>,
    adj_sw: Vec<R>,
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

        // Wall-bounded acceptance (the wall-bounded-ns capability): every
        // wall axis must carry at least two vertex layers (an extent-1 wall
        // axis has no 2-cells and no interior to march), and the metric's
        // grade-1 star must vend strictly positive, finite masses — the
        // operational form of "carries the boundary-corrected star" that
        // the constrained projection's masked CG normal form requires.
        let any_wall = complex.periodic().iter().any(|&p| !p);
        if any_wall {
            for (axis, (&periodic, &extent)) in complex
                .periodic()
                .iter()
                .zip(complex.shape().iter())
                .enumerate()
            {
                if !periodic && extent < 2 {
                    return Err(PhysicsError::DimensionMismatch(format!(
                        "DecNsRate: wall axis {axis} has extent {extent}; wall-bounded \
                         lattices need at least 2 vertex layers per wall axis"
                    )));
                }
            }
            use deep_causality_topology::HasHodgeStar;
            let metric = manifold
                .metric()
                // Coverage exemption: metric presence checked above.
                .expect("metric presence checked above");
            let star = metric
                .hodge_star_matrix(complex, 1)
                .map_err(|e| PhysicsError::TopologyError(format!("hodge star (grade 1): {e}")))?;
            for i in 0..n1 {
                let mut diag = R::zero();
                for e in star.row_indices()[i]..star.row_indices()[i + 1] {
                    if star.col_indices()[e] == i {
                        diag = star.values()[e];
                    }
                }
                if !diag.is_finite() || diag <= R::zero() {
                    return Err(PhysicsError::TopologyError(format!(
                        "DecNsRate: wall-bounded lattices require the boundary-corrected \
                         Hodge star with strictly positive edge masses; edge {i} has \
                         mass {diag}"
                    )));
                }
            }
        }

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
        let (adj_s1_len, adj_sw_len) = tables.convective_vector_adjoint_scratch_lens();
        let ws = RateWorkspace {
            omega: vec![R::zero(); n2],
            pre: vec![R::zero(); pre_len],
            wedge: vec![R::zero(); wedge_len],
            conv: vec![R::zero(); n1],
            visc_a: vec![R::zero(); n1],
            visc_b: vec![R::zero(); n1],
            s0: vec![R::zero(); n0],
            adj_corr: vec![R::zero(); n1],
            adj_s1: vec![R::zero(); adj_s1_len],
            adj_sw: vec![R::zero(); adj_sw_len],
        };
        let engine = Some(StencilEngine {
            tables,
            ws: RefCell::new(ws),
        });

        let no_slip = super::dec_ns_solver::no_slip::NoSlipConstraint::new(complex);

        Ok(Self {
            manifold,
            nu,
            body_force,
            n1,
            engine,
            spectral: None,
            no_slip,
        })
    }

    /// The wall-tangential edge set the no-slip condition constrains
    /// (empty on fully periodic lattices). Shared with the solver's seeding
    /// and re-entry projections.
    pub(in crate::theories::fluid_dynamics::dec) fn no_slip_edges(&self) -> &[usize] {
        self.no_slip.edges()
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

    /// The per-term energy budget of this rate at state `u` (the
    /// dec-ns-stability diagnostic): M-inner products of the state
    /// against the convective, viscous, and body-force terms (with the
    /// rate's signs) and against the projected rate. Evaluates through
    /// whichever assembly this rate is configured with (fused stencils by
    /// default, generic via [`Self::with_generic_assembly`]), so the
    /// budget can discriminate between the two strategies.
    ///
    /// # Errors
    /// `PhysicsError::TopologyError` when the projection solve does not
    /// converge within the supplied budget.
    pub fn energy_budget(
        &self,
        u: &VelocityOneForm<R>,
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<super::energy_budget::EnergyBudget<R>, PhysicsError> {
        let u_slice = u.as_tensor().as_slice();

        // Per-term vectors through the configured assembly. `conv` is
        // `i_u(du♭)` and `lap` is `Δ_dR u♭`; the rate carries them as
        // `−conv` and `−ν·lap`.
        let (conv, lap): (Vec<R>, Vec<R>) = if let Some(engine) = &self.engine {
            let t = &engine.tables;
            let mut ws = engine.ws.borrow_mut();
            let ws = &mut *ws;
            t.apply_d1(u_slice, &mut ws.omega)
                .expect("workspace lengths fixed at construction");
            Self::fill_convective_skew_fused(t, ws, u_slice);
            if let Some(spectral) = &self.spectral {
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
            let lap = ws
                .visc_a
                .iter()
                .zip(ws.visc_b.iter())
                .map(|(a, b)| *a + *b)
                .collect();
            (ws.conv.clone(), lap)
        } else {
            let conv = self.convective_skew_generic(u);
            let mut lap = self.manifold.laplacian_of(u_slice, 1).into_vec();
            lap.resize(self.n1, R::zero());
            (conv, lap)
        };

        // ⟨u, v⟩_M through the diagonal star (same form as the kinetic
        // energy diagnostic).
        let m_inner = |v: &[R]| -> R {
            let star_v = self.manifold.hodge_star_of(v, 1);
            u_slice
                .iter()
                .zip(star_v.as_slice().iter())
                .fold(R::zero(), |acc, (a, b)| acc + *a * *b)
        };

        let convective = R::zero() - m_inner(&conv);
        let viscous = R::zero() - self.nu * m_inner(&lap);
        let body_force = match &self.body_force {
            Some(g) => m_inner(g.as_slice()),
            None => R::zero(),
        };
        let projected_rate = self.eval_projected(u, opts)?;
        let projected = m_inner(projected_rate.as_tensor().as_slice());

        Ok(super::energy_budget::EnergyBudget {
            convective,
            viscous,
            body_force,
            projected,
        })
    }

    /// The shared projection of a raw RHS evaluation: constrained on
    /// wall-bounded lattices (the M-orthogonal projection onto no-slip ∩
    /// divergence-free), plain on periodic ones (the empty edge set
    /// delegates inside the topology call).
    fn project_raw(
        &self,
        raw: &VelocityOneForm<R>,
        opts: &HodgeDecomposeOptions<R>,
    ) -> Result<LerayProjection<R>, PhysicsError> {
        self.manifold
            .leray_project_constrained_opts(raw.as_tensor(), self.no_slip.edges(), opts)
            .map_err(|e| PhysicsError::TopologyError(format!("Leray projection failed: {e}")))
    }

    /// The skew-symmetrized convective term through the compiled tables
    /// (the dec-ns-stability fix): leaves
    /// `conv' = ½[G_ω u − G*_ω u]` in `ws.conv` (`ω = du` already in
    /// `ws.omega`; `G_ω` is the vector-slot map `x ↦ i_x ω`).
    /// `⟨u, conv'⟩_M = 0` identically, and the continuum antisymmetry
    /// `ω(x, w) = −ω(w, x)` makes the skew part full-strength consistent
    /// — the uncorrected gather alone injects energy in under-resolved
    /// turbulent regimes (measured 2026-06-12; see the
    /// fix-dec-convective-instability change).
    fn fill_convective_skew_fused(
        tables: &DecStencilTables<R>,
        ws: &mut RateWorkspace<R>,
        u_slice: &[R],
    ) {
        // Coverage exemptions on the unwraps: buffer lengths are fixed at
        // construction from the same tables.
        tables
            .apply_convective(&ws.omega, u_slice, &mut ws.pre, &mut ws.wedge, &mut ws.conv)
            .expect("workspace lengths fixed at construction");
        tables
            .apply_convective_vector_adjoint(
                &ws.pre,
                u_slice,
                &mut ws.adj_s1,
                &mut ws.adj_sw,
                &mut ws.adj_corr,
            )
            .expect("workspace lengths fixed at construction");
        let half = R::from_f64(0.5)
            // Coverage exemption: 0.5 lifts into every real field.
            .expect("0.5 lifts into R");
        for (c, k) in ws.conv.iter_mut().zip(ws.adj_corr.iter()) {
            *c = half * (*c - *k);
        }
    }

    /// The skew-symmetrized convective term through the generic operators
    /// — the equivalence oracle. The vector-slot adjoint `G*_ω` is
    /// assembled column by column through the public interior product
    /// (quadratic cost; test-scale lattices only, which is the generic
    /// path's role).
    fn convective_skew_generic(&self, u: &VelocityOneForm<R>) -> Vec<R> {
        let u_slice = u.as_tensor().as_slice();
        let du = self.manifold.exterior_derivative_of(u_slice, 1);
        let conv_raw = self
            .manifold
            .interior_product(u.as_tensor(), &du, 2)
            // Coverage exemption: grade (2 <= D) and operand lengths are
            // fixed by construction; interior_product cannot reject them.
            .expect("interior_product preconditions validated at construction")
            .into_vec();

        // Star diagonal through the public application on a unit cochain.
        let m1 = self.manifold.hodge_star_of(&vec![R::one(); self.n1], 1);
        let m1 = m1.as_slice();
        let zero_tol = <R as FromPrimitive>::from_f64(1e-12)
            // Coverage exemption: 1e-12 lifts into every real field.
            .expect("1e-12 is representable in every RealField");

        // G*_ω u: column j of G_ω is i_{e_j} ω;
        // (G*u)[j] = ⟨G e_j, M₁u⟩ / M₁[j].
        let w: Vec<R> = u_slice
            .iter()
            .zip(m1.iter())
            .map(|(a, b)| *a * *b)
            .collect();
        let mut adj = vec![R::zero(); self.n1];
        for (j, slot) in adj.iter_mut().enumerate() {
            let mut e = vec![R::zero(); self.n1];
            e[j] = R::one();
            let e_t = CausalTensor::new(e, vec![self.n1])
                // Coverage exemption: 1-D tensor allocation cannot fail.
                .expect("1-D tensor allocation cannot fail");
            let col = self
                .manifold
                .interior_product(&e_t, &du, 2)
                // Coverage exemption: as above.
                .expect("interior_product preconditions validated at construction");
            let dot = col
                .as_slice()
                .iter()
                .zip(w.iter())
                .fold(R::zero(), |acc, (a, b)| acc + *a * *b);
            *slot = if m1[j].abs() <= zero_tol {
                R::zero()
            } else {
                dot / m1[j]
            };
        }
        let half = R::from_f64(0.5)
            // Coverage exemption: 0.5 lifts into every real field.
            .expect("0.5 lifts into R");
        conv_raw
            .iter()
            .zip(adj.iter())
            .map(|(c, k)| half * (*c - *k))
            .collect()
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

        // The skew-symmetrized convective term (energy-neutral by
        // construction; see `convective_skew_generic`).
        let conv = self.convective_skew_generic(u);

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
        Self::fill_convective_skew_fused(t, ws, u_slice);
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
