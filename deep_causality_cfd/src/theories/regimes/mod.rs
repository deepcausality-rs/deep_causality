/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The classical pointwise Navier–Stokes regime evaluators — incompressible
//! Newtonian, Euler (inviscid), Stokes (creeping flow), and compressible
//! Newtonian — together with their causal (`PropagatingEffect`) wrappers.
//!
//! Migrated out of `deep_causality_physics::theories::fluid_dynamics`. Each
//! regime is a stateless, side-effect-free pointwise RHS that composes the
//! governing kernels imported from `deep_causality_physics`; the wrappers lift
//! each `Result` into the causal monad. These back the pointwise `FluidTheory`
//! adapters used by the verification solvers.

mod compressible_ns;
mod euler;
mod incompressible_ns;
mod stokes;
mod wrappers;

pub use compressible_ns::{
    compressible_ns_continuity_rhs_kernel, compressible_ns_energy_rhs_kernel,
    compressible_ns_momentum_rhs_kernel,
};
pub use euler::euler_momentum_rhs_kernel;
pub use incompressible_ns::incompressible_ns_rhs_kernel;
pub use stokes::stokes_momentum_rhs_kernel;
pub use wrappers::{
    compressible_ns_continuity_rhs, compressible_ns_energy_rhs, compressible_ns_momentum_rhs,
    euler_momentum_rhs, incompressible_ns_rhs, stokes_momentum_rhs,
};
