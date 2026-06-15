/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # deep_causality_cfd
//!
//! Computational fluid dynamics solvers and the **Flow** DSL for DeepCausality.
//!
//! This crate consolidates the fluid-dynamics theories and the DEC-native
//! Navier–Stokes solver behind a composable, precision-generic interface,
//! and lifts them into the `Flow` domain-specific language
//!
//! Physics errors (`PhysicsError`), physics quantities (the typed DEC forms and
//! quantity newtypes), and the pointwise governing kernels stay consolidated in
//! `deep_causality_physics`; this crate imports them rather than duplicating them.
//!
//! Precision is a parameter: every theory and solver is generic over a real
//! scalar (`CfdScalar`). Composition is static (no `dyn`),
//! built on the `deep_causality_haft` HKT/algebra foundation.
//!
//! CPU parallelism is opt-in via the `parallel` feature
//! and rides the `MaybeParallel` bound.

extern crate alloc;

mod errors;
mod extensions;
mod solvers;
mod theories;
mod traits;
mod types;

// Physics types this crate's public API exposes — the typed DEC forms and
// physics-quantity newtypes (which stay in `deep_causality_physics`) plus
// `PhysicsError` — re-exported so CFD code can import them from one crate.
pub use deep_causality_physics::PhysicsError;
pub use deep_causality_physics::quantities::*;

// Core CFD trait seams and value types.
pub use crate::traits::{FluidTheory, Marcher};
pub use crate::types::{Ambient, CfdScalar};

// The Flow DSL facade (owned case descriptions materialized at run).
pub use crate::types::flow::{Flow, MarchBuilder, MarchCase, Mesh, Observe, Report, Seed};

// Fluid-dynamics theories: the DEC-native FluidTheory realization, and the
// classical pointwise NS regime evaluators + their causal wrappers.
pub use crate::theories::DecIncompressible;
pub use crate::theories::regimes::*;

// Solver configuration + type-state builder.
pub use crate::solvers::{
    DecNs, DecNsConfig, DecNsConfigNeedsTimeStep, DecNsConfigNeedsViscosity, DecNsConfigReady,
};

// Public API of the Navier–Stokes solver.
pub use crate::solvers::dec::*;
