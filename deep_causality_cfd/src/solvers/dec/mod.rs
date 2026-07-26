/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The periodic DEC-native incompressible Navier–Stokes solver.
//!
//! Velocity is an edge 1-form for the entire solve. The governing
//! formulation is the rotational (Lamb) form under Leray projection:
//!
//! ```text
//! ∂u♭/∂t = P( − ½[i_u ω − G*_ω u] − ν Δ_dR u♭ + g♭ ),   ω = d u♭
//! ```
//!
//! Every symbol here is the operator the code applies. The viscous term is
//! written with the **same** `Δ_dR` the solver evaluates: on a flat torus the
//! Hodge–de Rham Laplacian satisfies `Δ_dR = −∇²`, so `− ν Δ_dR u♭` *is* the
//! physical diffusion `+ ν ∇² u♭`. The convective term is skew-symmetrized for
//! stability (`G*_ω` is the M-adjoint of `x ↦ i_x ω`; the dec-ns-stability
//! fix); see [`DecNsRate`].
//!
//! Pressure vanishes from the equations (the projector annihilates every
//! gradient, including `∇(|u|²/2)` of the Lamb split), and
//! incompressibility is exact by construction: the projected field's
//! discrete divergence is zero to the CG tolerance — the projector *is*
//! the equation.
//!
//! The march applies the Leray projector **inside each `Rk4` stage**: the
//! stage rate is the projected rate `P∘rhs`, so every increment is
//! divergence-free and the marched ODE is the projected dynamics with no
//! operator-splitting error. A near-free type-state re-entry projection
//! follows (its input is already divergence-free), restoring the
//! [`SolenoidalField`](deep_causality_physics::SolenoidalField) construction
//! contract; the CFL guard runs last. Time accuracy is the integrator's; the
//! spatial discretisation dominates the error, so validation gates on spatial
//! refinement at fixed CFL.
//!
//! Module layout:
//! - [`DecNsRate`]: the right-hand side `−½[i_u(du♭) − G*_ω u] − ν Δ_dR u♭ + g♭`,
//!   with the skew-symmetrized convective term, validated at construction so
//!   per-step evaluation is infallible and composes directly with
//!   `deep_causality_calculus::Rk4`.
//! - [`DecNsSolver`]: configuration, the projected step, run loops,
//!   initial-condition seeding, and the opt-in pressure diagnostic.
//! - [`StepOutput`] / [`RunOutput`]: per-step and per-run results carrying
//!   the diagnostics the step already computed.
//! - [`diagnostics`]: DEC-native integral observables (energy, enstrophy,
//!   helicity, max speed, divergence residual).
//! - [`wrappers`]: the `PropagatingEffect` surface in the crate's existing
//!   kernel-wrapper tradition.
//!
//! Method reference: Mohamed, Hirani & Samtaney, "Discrete exterior calculus discretization of
//! incompressible Navier–Stokes equations over surface simplicial meshes", Journal of Computational
//! Physics 312:175–191, 2016 (`deep_causality_cfd/papers/mohamed2016.pdf`). This solver follows that
//! DEC formulation on a periodic lattice complex rather than a surface simplicial mesh.

use core::fmt::{Debug, Display};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;

/// The composed bound set of the DEC solver stack: the topology operators
/// require `RealField + Default + PartialEq + Debug (+ FromPrimitive)`,
/// the typed-form constructors add `Display`, `Rk4`'s `Scalar` is
/// satisfied by `RealField + FromPrimitive` through the blanket impl, and
/// `MaybeParallel` carries the topology crate's `parallel`-feature
/// thread-safety requirement (vacuous on serial builds; `Send + Sync`
/// under `--features parallel` — every workspace scalar qualifies).
pub trait DecNsScalar:
    RealField + FromPrimitive + Default + PartialEq + Debug + Display + MaybeParallel
{
}
impl<R: RealField + FromPrimitive + Default + PartialEq + Debug + Display + MaybeParallel>
    DecNsScalar for R
{
}

pub(crate) mod boundary;
pub(crate) mod dec_ns_rate;
pub(crate) mod dec_ns_solver;
pub(crate) mod diagnostics;
// cfd-native: the `Marcher` trait realization for `DecNsSolver`.
pub(crate) mod energy_budget;
mod marcher;
pub(crate) mod scalar_transport;
pub(crate) mod spectral_diffusion;
pub(crate) mod step_output;
pub(crate) mod surface_force;
// The uncertain-inflow zone (Group C) consumes `deep_causality_uncertain` and its global sample
// cache, which is std-only.
#[cfg(feature = "std")]
pub(crate) mod uncertain_inflow;
pub(crate) mod wrappers;
// Owned configuration + type-state builder for the DEC solver (design D2).
pub mod dec_config;

pub use boundary::{BodyForceZone, BoundaryZone, Inflow, MovingWall, Outflow, SlipWall};
pub use dec_ns_rate::DecNsRate;
pub use dec_ns_solver::DecNsSolver;
pub use diagnostics::{
    dec_divergence_residual, dec_enstrophy, dec_helicity, dec_kinetic_energy, dec_max_speed,
    dec_sample_velocity,
};
pub use energy_budget::EnergyBudget;
pub use scalar_transport::DecScalarRate;
pub use step_output::{RunOutput, StepOutput};
pub use surface_force::{
    force_coefficient, fragment_area_vector, pressure_surface_force, viscous_surface_force,
    wall_heat_flux,
};
#[cfg(feature = "std")]
pub use uncertain_inflow::{
    DropoutVerbosity, InflowContext, InflowMarchState, InflowProcess, UncertainBoundarySource,
    UncertainInflowZone, inflow_march_step, march_inflow,
};
pub use wrappers::dec_ns_step;
